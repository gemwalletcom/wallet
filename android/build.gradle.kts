buildscript {
    repositories {
        gradlePluginPortal()
        google()
        mavenCentral()
    }
    dependencies {
        classpath(libs.gradle)
        classpath(libs.hilt.android.gradle.plugin)
        classpath(libs.kotlin.serialization)
        constraints {
            listOf(
                "org.bouncycastle:bcprov-jdk18on:1.85",
                "org.bouncycastle:bcpkix-jdk18on:1.85",
                "org.bouncycastle:bcutil-jdk18on:1.85",
                "com.google.guava:guava:33.7.2-jre",
                "org.bitbucket.b_c:jose4j:0.9.6",
                "org.jdom:jdom2:2.0.6.1",
                "org.apache.commons:commons-lang3:3.18.0",
            ).forEach { classpath(it) { because("Android Gradle plugin resolves a version with published advisories") } }
        }
    }
    configurations.classpath {
        resolutionStrategy.activateDependencyLocking()
    }
    dependencyLocking {
        lockMode = LockMode.STRICT
    }
}

plugins {
    alias(libs.plugins.ksp) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.google.services) apply false
    alias(libs.plugins.compose.compiler) apply false
}

val gemstoneHostLibrary = File(rootDir, "../core/target/debug/${System.mapLibraryName("gemstone")}")
extra["gemstoneHostLibrary"] = gemstoneHostLibrary

allprojects {
    repositories {
        google()
        mavenCentral()
    }
}

subprojects {
    tasks.withType<Test>().configureEach {
        dependsOn(":gemstone:buildGemstoneHost")
        inputs.file(gemstoneHostLibrary)
            .withPropertyName("gemstoneHostLibrary")
            .withPathSensitivity(PathSensitivity.NONE)
        systemProperty("jna.library.path", gemstoneHostLibrary.parentFile.absolutePath)
        jvmArgs("-XX:TieredStopAtLevel=1", "-XX:+UseSerialGC")
        when (providers.gradleProperty("integrationTests").orNull) {
            "skip" -> filter {
                excludeTestsMatching("*.integration.*")
                isFailOnNoMatchingTests = false
            }
            "only" -> filter {
                includeTestsMatching("*.integration.*")
                isFailOnNoMatchingTests = false
            }
        }
    }
    tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinJvmCompile>().configureEach {
        compilerOptions.allWarningsAsErrors.set(true)
    }
    if (path != ":app" && file("src/androidTest").exists()) {
        throw GradleException("$path has src/androidTest. Put tests that need Android in an integration package under src/test; they run on Robolectric.")
    }
    plugins.withId("com.android.library") {
        extensions.configure<com.android.build.api.dsl.LibraryExtension> {
            compileSdk {
                version = release(37) { minorApiLevel = 1 }
            }
            defaultConfig {
                minSdk = 28
                testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
            }
            compileOptions {
                sourceCompatibility = JavaVersion.VERSION_17
                targetCompatibility = JavaVersion.VERSION_17
            }
            packaging {
                resources {
                    excludes += "META-INF/*"
                    excludes += "META-INF/DEPENDENCIES"
                    excludes += "/META-INF/LICENSE-notice.md"
                    excludes += "/META-INF/LICENSE.md"
                    excludes += "META-INF/versions/9/OSGI-INF/MANIFEST.MF"
                }
            }
        }
        tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinJvmCompile>().configureEach {
            compilerOptions {
                jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
                freeCompilerArgs.add("-opt-in=kotlin.RequiresOptIn")
            }
        }
    }
    plugins.withId("org.jetbrains.kotlin.plugin.compose") {
        extensions.configure<org.jetbrains.kotlin.compose.compiler.gradle.ComposeCompilerGradlePluginExtension> {
            stabilityConfigurationFiles.add(rootProject.layout.projectDirectory.file("compose_stability.conf"))
        }
    }
    listOf("com.android.library", "com.android.application").forEach { pluginId ->
        plugins.withId(pluginId) {
            dependencies.add("testImplementation", libs.jna)
            dependencies.add("testImplementation", libs.robolectric)
            dependencies.add("testImplementation", libs.androidx.junit)
            extensions.getByType(com.android.build.api.dsl.CommonExtension::class.java).sourceSets.getByName("test").resources.directories.add(rootProject.file("gradle/robolectric").path)
        }
    }
}

tasks.register("clean", Delete::class) {
    delete(layout.buildDirectory)
}
