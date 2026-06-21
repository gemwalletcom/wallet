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

val fdroidBuild = System.getenv("FDROID_BUILD") == "true"

allprojects {
    repositories {
        val propFile = File(rootDir.absolutePath, "local.properties")
        var properties = java.util.Properties()
        if (propFile.exists()) {
            properties = properties.apply {
                propFile.inputStream().use { fis ->
                    load(fis)
                }
            }
        }
        google()
        mavenCentral()
        maven { url = uri("https://jitpack.io") }
    }

    dependencyLocking {
        lockAllConfigurations()
    }
}

subprojects {
    dependencyLocking {
        lockAllConfigurations()
    }
    configurations.configureEach {
        if (fdroidBuild) {
            exclude(group = "com.google.firebase")
            exclude(group = "com.google.android.gms")
        }
        resolutionStrategy.activateDependencyLocking()
    }
}

tasks.register("clean", Delete::class) {
    delete(layout.buildDirectory)
}
