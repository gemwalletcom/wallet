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
    }
}

plugins {
    alias(libs.plugins.ksp) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.google.services) apply false
    alias(libs.plugins.room) apply false
    alias(libs.plugins.compose.compiler) apply false
}

val gemstoneHostLibrary by extra(File(rootDir, "../core/target/debug/${System.mapLibraryName("gemstone")}"))

allprojects {
    repositories {
        google()
        mavenCentral()
        maven { url = uri("https://jitpack.io") }
    }

    dependencyLocking {
        lockAllConfigurations()
    }
}

subprojects {
    configurations.configureEach {
        resolutionStrategy.activateDependencyLocking()
    }
    tasks.withType<Test>().configureEach {
        dependsOn(":gemstone:buildGemstoneHost")
        inputs.file(gemstoneHostLibrary)
            .withPropertyName("gemstoneHostLibrary")
            .withPathSensitivity(PathSensitivity.NONE)
        systemProperty("jna.library.path", gemstoneHostLibrary.parentFile.absolutePath)
        when (providers.gradleProperty("integrationTests").orNull) {
            "skip" -> exclude("**/integration/**")
            "only" -> filter {
                includeTestsMatching("*.integration.*")
                isFailOnNoMatchingTests = false
            }
        }
    }
    if (path != ":app" && file("src/androidTest").exists()) {
        throw GradleException("$path has src/androidTest. Put tests that need Android in an integration package under src/test; they run on Robolectric.")
    }
    listOf("com.android.library", "com.android.application").forEach { pluginId ->
        plugins.withId(pluginId) {
            dependencies.add("testImplementation", "net.java.dev.jna:jna:5.18.1")
            dependencies.add("testImplementation", libs.robolectric)
            dependencies.add("testImplementation", libs.androidx.junit)
            extensions.getByType(com.android.build.api.dsl.CommonExtension::class.java).sourceSets.getByName("test").resources.directories.add(rootProject.file("gradle/robolectric").path)
        }
    }
}

tasks.register("clean", Delete::class) {
    delete(layout.buildDirectory)
}
