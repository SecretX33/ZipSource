# Gradle

In these build gradle files (both Groovy and KTS), I've replicated the use-case that inspired me to create ZipSource in the first place, and the configuration that solved it.

## Tasks
There are three tasks:

- `createZipJars`: uses Shadow plugin to create the `.jar` output files, then zip them.
- `createZipSource`: uses ZipSource to create the source code `.zip` file.
- `dist`: convenience task that executes all tasks above in a single run.

## How to use

After copying the proper `zipsource` executable (according to your OS) to `tools` folder in the root project folder (so the final path becomes `tools/zipsource`), the project source code zip file plus zipped binaries can be created simply by running:

```shell
./gradlew dist
```