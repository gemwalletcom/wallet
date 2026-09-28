plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.fiat_connect.presents"

    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(project(":ui"))
    implementation(project(":features:fiat_connect:viewmodels"))

    implementation(libs.hilt.lifecycle.viewmodel.compose)

    testImplementation(libs.junit)
    testImplementation(testFixtures(project(":gemcore")))
}
