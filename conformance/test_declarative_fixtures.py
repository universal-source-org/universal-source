"""Check fixture data and references. Never dispatch or execute source operations."""

from pathlib import Path
import unittest
from urllib.parse import urlsplit

from jsonschema import Draft202012Validator

from test_manifests import ROOT, SCHEMA, read_json


DIRECTORY = ROOT / "conformance/declarative"


class DeclarativeFixtureConsistency(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.schema = read_json(DIRECTORY / "cases.schema.json")
        Draft202012Validator.check_schema(cls.schema)
        cls.validator = Draft202012Validator(cls.schema)
        cls.corpus = read_json(DIRECTORY / "cases.json")
        cls.validator.validate(cls.corpus)
        manifest_schema = read_json(SCHEMA)
        Draft202012Validator.check_schema(manifest_schema)
        cls.manifest_validator = Draft202012Validator(manifest_schema)

    def contained_file(self, base, relative):
        self.assertFalse(Path(relative).is_absolute())
        path = (base / relative).resolve()
        self.assertTrue(path.is_relative_to(base.resolve()), relative)
        self.assertTrue(path.is_file(), relative)
        return path

    def sources(self):
        sources = {}
        for name, context in self.corpus["sources"].items():
            manifest_path = self.contained_file(ROOT, context["manifest"])
            manifest = read_json(manifest_path)
            self.manifest_validator.validate(manifest)
            self.assertEqual(manifest["engine"], "declarative")
            entry_path = self.contained_file(manifest_path.parent, manifest["entry"])
            entry = read_json(entry_path)
            # Validate stored shapes against the fixture aid, not a runtime loader.
            self.validator.evolve(schema={
                "$ref": "#/$defs/entry", "$defs": self.schema["$defs"]
            }).validate(entry)
            self.assertEqual(set(entry), set(manifest["capabilities"]["operations"]))
            sources[name] = (manifest, entry)
        return sources

    def assert_unique_ids(self, records):
        ids = [record["id"] for record in records]
        self.assertEqual(len(ids), len(set(ids)))

    def assert_granted_urls(self, value, origins):
        # Only inspect URL-bearing fields. Opaque IDs and descriptions are not URLs.
        if isinstance(value, dict):
            for key, item in value.items():
                if key in {"url", "poster"}:
                    url = urlsplit(item)
                    self.assertIn(url.scheme, {"http", "https"})
                    self.assertTrue(url.hostname)
                    self.assertIsNone(url.username)
                    self.assertIsNone(url.password)
                    self.assertIn(f"{url.scheme}://{url.netloc}", origins)
                else:
                    self.assert_granted_urls(item, origins)
        elif isinstance(value, list):
            for item in value:
                self.assert_granted_urls(item, origins)

    def test_case_identity_and_grants(self):
        sources = self.sources()
        cases = self.corpus["cases"]
        ids = [case["id"] for case in cases]
        self.assertEqual(len(ids), len(set(ids)), "Duplicate case ID")
        self.assertEqual({case["source"] for case in cases}, set(sources))
        self.assertEqual({case["operation"] for case in cases},
                         {"home", "category", "search", "detail", "play"})
        for case in cases:
            with self.subTest(case=case["id"]):
                manifest, _ = sources[case["source"]]
                requested = manifest["permissions"]
                granted = case["grants"]
                self.assertTrue(set(granted["network"]) <= set(requested["network"]))
                for flag in ("cookies", "storage"):
                    self.assertFalse(granted[flag] and not requested[flag])
                declared = case["operation"] in manifest["capabilities"]["operations"]
                self.assertEqual(
                    case["expect"].get("errorCode") == "UNSUPPORTED_OPERATION",
                    not declared,
                    "Undeclared-operation expectation must agree with the manifest",
                )

    def test_stored_source_references(self):
        for name, (manifest, entry) in self.sources().items():
            with self.subTest(source=name):
                home = entry.get("home", {"categories": [], "items": []})
                self.assert_unique_ids(home["categories"])
                if "category" in entry:
                    for category in home["categories"]:
                        self.assertIn(category["id"], entry["category"])
                lists = [home["items"]]
                lists += list(entry.get("category", {}).values())
                lists += list(entry.get("search", {}).values())
                for records in lists:
                    self.assert_unique_ids(records)
                    for record in records:
                        if "detail" in entry:
                            self.assertIn(record["id"], entry["detail"])
                playables = {}
                for key, detail in entry.get("detail", {}).items():
                    self.assertEqual(key, detail["id"])
                    self.assert_unique_ids(detail["playables"])
                    for playable in detail["playables"]:
                        if "play" in entry:
                            self.assertIn(playable["id"], entry["play"])
                        if playable["id"] in playables:
                            self.assertEqual(playable, playables[playable["id"]])
                        playables[playable["id"]] = playable
                for operation in ("category", "search", "detail", "play"):
                    for key in entry.get(operation, {}):
                        self.assertTrue(key)
                        if operation == "search":
                            self.assertTrue(key.strip(" \t\r\n"))
                # Current contexts use canonical exact origins; no URL is fetched.
                for operation, stored in entry.items():
                    # Lookup-map keys are opaque IDs/queries, even if named "url".
                    records = stored if operation == "home" else list(stored.values())
                    self.assert_granted_urls(records, manifest["permissions"]["network"])

    def test_expected_success_snapshots(self):
        sources = self.sources()
        for case in self.corpus["cases"]:
            if "result" not in case["expect"]:
                continue
            with self.subTest(case=case["id"]):
                _, entry = sources[case["source"]]
                data = case["expect"]["result"]["data"]
                operation = case["operation"]
                self.assert_granted_urls(data, case["grants"]["network"])
                # Compare authored snapshots to stored data, without predicting
                # which outcome an input should produce or running any dispatcher.
                if operation == "home":
                    self.assertEqual(data, entry["home"])
                elif operation == "detail":
                    self.assertEqual(data["id"], case["input"]["id"])
                    self.assertEqual(data, entry["detail"][data["id"]])
                elif operation == "play":
                    self.assertEqual(data, entry["play"][case["input"]["id"]])
                else:
                    self.assertEqual(data["page"], case["input"].get("page", 1))
                    self.assert_unique_ids(data["items"])
                    if data["items"]:
                        self.assertIn(data["items"], entry[operation].values())


if __name__ == "__main__":
    unittest.main()
