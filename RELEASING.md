# Releasing Flexweave

Flexweave releases are published from the `Release` GitHub Actions workflow.
The workflow updates the workspace version, verifies and packages the crate,
publishes it to crates.io, tags the release commit, and creates the corresponding
GitHub Release with automatically generated notes.

The documentation site is deployed independently by Vercel and is not part of
the crate release workflow.

## One-time setup

Configure a [crates.io trusted publisher](https://crates.io/docs/trusted-publishing)
for the `flexweave` crate with these values:

- GitHub owner: `RickyRoller`
- GitHub repository: `flexweave`
- Workflow file: `release.yml`
- Environment: `release`

The workflow uses OpenID Connect to obtain a short-lived crates.io token, so no
long-lived crates.io token needs to be stored in GitHub.

The `release` GitHub environment is also the place to add required reviewers if
publishing should require an approval after the workflow is started.

## Cut a release

1. Open **Actions**, select **Release**, and choose **Run workflow**.
2. Keep **Use workflow from** set to `main`, enter a stable semantic version
   without a `v` prefix (for example, `0.2.0`), and run the workflow.

If a run fails after updating `main`, run it again with the same version. The
workflow verifies an existing crates.io package by checksum and safely resumes
the remaining tag and GitHub Release steps.
