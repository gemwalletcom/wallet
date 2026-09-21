import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.library")
}

val gemstoneRoot = rootProject.projectDir.resolve("../core/gemstone")
val coreRoot = gemstoneRoot.parentFile
val gemstoneSrc = gemstoneRoot.resolve("android/gemstone/src")
val jniLibsDir = gemstoneSrc.resolve("main/jniLibs")
val generatedKotlinDir = gemstoneSrc.resolve("main/java")
val isRelease = System.getenv("BUILD_MODE") == "release"
val cargoBuildFlag = if (isRelease) "--release" else ""
val gemstoneHostLibrary: File by rootProject.extra
val defaultCargoNdkAbis = when {
    System.getenv("UNIT_TESTS") == "true" -> "x86_64"
    isRelease -> "arm64-v8a,armeabi-v7a"
    else -> "arm64-v8a"
}
val cargoNdkTargets = (System.getenv("GEMSTONE_ANDROID_ABIS") ?: defaultCargoNdkAbis)
    .split(",")
    .map { it.trim() }
    .filter { it.isNotEmpty() }
    .joinToString(" ") { "-t $it" }

android {
    namespace = "com.gemwallet.gemstone"
    compileSdk = 37
    ndkVersion = libs.versions.androidNdk.get()

    defaultConfig {
        minSdk = 28
        consumerProguardFiles(gemstoneRoot.resolve("android/gemstone/consumer-rules.pro"))
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets {
        getByName("main") {
            kotlin {
                directories.add(generatedKotlinDir.absolutePath)
            }
            jniLibs {
                directories.add(jniLibsDir.absolutePath)
            }
            manifest.srcFile(gemstoneSrc.resolve("main/AndroidManifest.xml"))
        }
        getByName("androidTest") {
            kotlin {
                directories.add(gemstoneSrc.resolve("androidTest/java").absolutePath)
            }
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}

tasks.withType<Exec>().configureEach {
    environment("CARGO_TARGET_DIR", coreRoot.resolve("target").absolutePath)
}

val buildGemstoneHost = tasks.register<Exec>("buildGemstoneHost") {
    description = "Build the host Gemstone library used by JVM tests"
    workingDir = coreRoot
    outputs.file(gemstoneHostLibrary)
    outputs.upToDateWhen { false }
    commandLine("/bin/sh", "-l", "-c", "cargo build --package gemstone --lib")
}

val bindgenKotlin = tasks.register<Exec>("bindgenKotlin") {
    description = "Generate Kotlin bindings from gemstone via uniffi"
    workingDir = gemstoneRoot
    if (isRelease) {
        outputs.upToDateWhen { false }
    } else {
        dependsOn(buildGemstoneHost)
        inputs.file(gemstoneHostLibrary)
    }
    inputs.dir(coreRoot.resolve("bin/uniffi-bindgen"))
    inputs.files(
        coreRoot.resolve("Cargo.toml"),
        coreRoot.resolve("Cargo.lock"),
        coreRoot.resolve(".cargo/config.toml"),
        coreRoot.parentFile.resolve("rust-toolchain.toml"),
        gemstoneRoot.resolve("uniffi.toml"),
        gemstoneRoot.resolve("justfile"),
    )
    inputs.property("cargoBuildFlag", cargoBuildFlag)
    outputs.dir(generatedKotlinDir.resolve("uniffi"))
    commandLine("/bin/sh", "-l", "-c", "just bindgen-kotlin")
}

val buildCargoNdk = tasks.register<Exec>("buildCargoNdk") {
    description = "Build gemstone native libraries using cargo-ndk"
    workingDir = gemstoneRoot
    outputs.dir(jniLibsDir)
    outputs.upToDateWhen { false }
    commandLine("/bin/sh", "-l", "-c", "cargo ndk $cargoNdkTargets -o ${jniLibsDir.absolutePath} build --lib $cargoBuildFlag")
}

tasks.configureEach {
    if (name.startsWith("lint") || name.startsWith("updateLintBaseline")) {
        enabled = false
    }
    if (name.matches(Regex("(compile|extract|source|javaDoc).*(Debug|Release).*"))) {
        dependsOn(bindgenKotlin)
    }
    if (name.matches(Regex("merge.*(Debug|Release).*JniLib.*"))) {
        dependsOn(buildCargoNdk)
    }
}

dependencies {
    api("net.java.dev.jna:jna:5.18.1@aar")
    implementation(libs.kotlinx.coroutines.android)
    androidTestImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
}
