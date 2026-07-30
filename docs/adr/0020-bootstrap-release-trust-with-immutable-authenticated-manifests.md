# Bootstrap release trust with immutable authenticated manifests

Status: Accepted

Every release, including the Validated MVP Linux binary, will use immutable assets and an authenticated digest manifest whose trust anchor is independent of the downloaded asset location; adjacent checksums from the same mutable release are insufficient. Full V1 binds the manifest to the pinned Action revision, publishes provenance and SBOM subjects for exact archives and binaries, and pins release-workflow actions by full commit SHA. This makes the native-binary speed and convenience promise compatible with a reviewable supply-chain trust bootstrap.
