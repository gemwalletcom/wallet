import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.tasks.KotlinJvmCompile

plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.service.store"
    compileSdk = 37

    defaultConfig {
        minSdk = 28

        consumerProguardFiles("consumer-rules.pro")

        ksp {
            arg("room.schemaLocation", "$projectDir/schemas")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    tasks.withType<KotlinJvmCompile>().configureEach {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
            freeCompilerArgs.add("-opt-in=kotlin.RequiresOptIn")
        }
    }
    testFixtures {
        enable = true
    }

    testOptions {
        unitTests {
            isIncludeAndroidResources = true
        }
    }

    sourceSets {
        getByName("test") {
            assets {
                directories.add("$projectDir/schemas")
            }
        }
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

dependencies {
    implementation(project(":gemcore"))
    testFixturesImplementation(project(":gemcore"))
    testFixturesImplementation(testFixtures(project(":gemcore")))

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    ksp(libs.room.compiler)
    implementation(libs.room.runtime)

    implementation(libs.ktx.core)
    testImplementation(libs.junit)
    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(libs.room.testing)
}
