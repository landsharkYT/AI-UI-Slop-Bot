#!/usr/bin/env python3
import datetime
import json
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
destination = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("ai-ui-slop.spdx.json")
lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
root_package = next(package for package in lock["package"] if package["name"] == "ai-ui-slop")
version = root_package["version"]

packages = []
for package in sorted(lock["package"], key=lambda item: (item["name"], item["version"])):
    safe_name = "".join(
        character if character.isalnum() or character in ".-" else "-"
        for character in package["name"]
    )
    packages.append(
        {
            "SPDXID": f"SPDXRef-Package-{safe_name}-{package['version']}",
            "name": package["name"],
            "versionInfo": package["version"],
            "downloadLocation": "NOASSERTION",
            "filesAnalyzed": False,
            "licenseConcluded": "NOASSERTION",
            "licenseDeclared": "NOASSERTION",
            "copyrightText": "NOASSERTION",
            "checksums": (
                [{"algorithm": "SHA256", "checksumValue": package["checksum"]}]
                if "checksum" in package
                else []
            ),
        }
    )

document = {
    "spdxVersion": "SPDX-2.3",
    "dataLicense": "CC0-1.0",
    "SPDXID": "SPDXRef-DOCUMENT",
    "name": f"ai-ui-slop-{version}",
    "documentNamespace": f"https://ai-ui-slop.dev/spdx/ai-ui-slop-{version}",
    "creationInfo": {
        "created": datetime.datetime.now(datetime.UTC)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "creators": ["Tool: ai-ui-slop/scripts/generate-sbom.py"],
    },
    "packages": packages,
}
destination.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
