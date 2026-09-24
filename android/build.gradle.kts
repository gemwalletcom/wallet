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
        if (providers.gradleProperty("skipIntegrationTests").isPresent) {
            exclude("**/integration/**")
        }
    }
    listOf("com.android.library", "com.android.application").forEach { pluginId ->
        plugins.withId(pluginId) {
            dependencies.add("testImplementation", "net.java.dev.jna:jna:5.18.1")
        }
    }
}

tasks.register("clean", Delete::class) {
    delete(layout.buildDirectory)
}
