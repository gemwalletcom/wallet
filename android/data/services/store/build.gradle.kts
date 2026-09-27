plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.data.services.store"

    defaultConfig {
        ksp {
            arg("room.schemaLocation", "$projectDir/schemas")
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
