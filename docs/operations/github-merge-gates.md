# GitHub Merge Gates

Kerald requires CI to pass before changes can merge into `main`.

Direct pushes to `main` are forbidden. All changes to `main` must enter through pull requests and must be merged with merge commits, not squash merges or rebase merges.

The desired repository ruleset is tracked in `.github/rulesets/main-required-ci.json`. GitHub does not automatically apply ruleset JSON stored in a repository, so repository administrators must create or update the GitHub ruleset through the GitHub API or UI.

To inspect existing repository rulesets:

```sh
gh api repos/Wouittone/Kerald/rulesets
```

To create the tracked ruleset:

```sh
gh api \
  --method POST \
  repos/Wouittone/Kerald/rulesets \
  --input .github/rulesets/main-required-ci.json
```

If `main-required-ci` already exists, update that ruleset instead of creating a duplicate:

```sh
gh api \
  --method PUT \
  repos/Wouittone/Kerald/rulesets/RULESET_ID \
  --input .github/rulesets/main-required-ci.json
```

The required status check is `ci / required`. The individual language and container build jobs are diagnostic checks; they are allowed to skip until the matching project files exist.

To align repository merge buttons with the `main` ruleset, keep merge commits enabled and disable squash and rebase merges:

```sh
gh api \
  --method PATCH \
  repos/Wouittone/Kerald \
  -f allow_merge_commit=true \
  -f allow_squash_merge=false \
  -f allow_rebase_merge=false
```
