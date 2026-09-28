plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.onboarding.viewmodels"
}

dependencies {
    implementation(project(":gemcore"))
    implementation(project(":ui"))
    api(project(":ui-models"))

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    implementation(libs.lifecycle.viewmodel)

    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(libs.junit)
    testImplementation(libs.mockk.android)
    testImplementation(libs.kotlinx.coroutines.test)
}
