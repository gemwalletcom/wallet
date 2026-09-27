plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.data.services.gemstone"

    testOptions {
        unitTests {
            isReturnDefaultValues = true
        }
    }
    buildFeatures {
        buildConfig = true
    }
}

dependencies {
    implementation(project(":data:services:store"))
    api(project(":gemcore"))
    implementation(libs.okhttp)

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    implementation(libs.datastore)

    implementation(libs.androidx.biometric)
    implementation(libs.ktx.core)
    testImplementation(testFixtures(project(":data:services:store")))
    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk.android)
    testImplementation(libs.room.runtime)
}
