plugins {
    //...
    id("com.github.johnrengelman.shadow") version "7.1.2"
}

group = "com.example"
version = "0.0.1"

repositories {
    //...
}

dependencies {
    //...
}

// Creates a .zip file with the project jars
val createZipJars by tasks.registering(Zip::class) {
    dependsOn(":shadowJar")
    group = "distribution"

    archiveFileName.set("${rootProject.name}.zip")
    destinationDirectory.set(file("$projectDir"))

    include("*.jar")
    fileTree(buildDir.resolve("libs")).forEach(::from)
}

// Creates a .zip with the project source code
val createZipSource by tasks.registering(Exec::class) {
    group = "distribution"
    workingDir(projectDir)
    commandLine("tools/zipsource")
}

tasks.register("dist") {
    dependsOn(createZipJars, createZipSource)
    group = "distribution"
}