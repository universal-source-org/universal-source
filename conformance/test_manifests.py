"""Schema conformance checks only: no source loading, execution, or network access."""

import json
from pathlib import Path
import unittest

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "spec/schema/manifest.schema.json"
FIXTURES = ROOT / "conformance"
EXAMPLE = ROOT / "examples/json/minimal/manifest.json"
DIALECT = "https://json-schema.org/draft/2020-12/schema"


def unique_members(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicate JSON member: {key!r}")
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError(f"Not a JSON number: {value}")


def read_json(path):
    # Parser errors must fail the suite, including for schema-invalid fixtures.
    return json.loads(
        path.read_text(encoding="utf-8"),
        object_pairs_hook=unique_members,
        parse_constant=reject_constant,
    )


class ManifestConformance(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        schema = read_json(SCHEMA)
        if schema.get("$schema") != DIALECT:
            raise ValueError(f"Expected the Draft 2020-12 schema dialect: {SCHEMA}")
        # Check the actual schema against the validator's bundled metaschema.
        Draft202012Validator.check_schema(schema)
        cls.validator = Draft202012Validator(schema)

    def fixture_paths(self, outcome):
        directory = FIXTURES / outcome / "manifests"
        paths = sorted(directory.glob("*.json"))
        self.assertTrue(paths, f"Missing or empty fixture directory: {directory}")
        return paths

    def test_valid_manifests(self):
        for path in self.fixture_paths("valid"):
            with self.subTest(fixture=path.relative_to(ROOT).as_posix()):
                self.validator.validate(read_json(path))

    def test_invalid_manifests(self):
        for path in self.fixture_paths("invalid"):
            with self.subTest(fixture=path.relative_to(ROOT).as_posix()):
                errors = list(self.validator.iter_errors(read_json(path)))
                self.assertTrue(errors, "Expected schema rejection, but manifest passed")

    def test_minimal_example_manifest(self):
        self.validator.validate(read_json(EXAMPLE))


if __name__ == "__main__":
    unittest.main()
