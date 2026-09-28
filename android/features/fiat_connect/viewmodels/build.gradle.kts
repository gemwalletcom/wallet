plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.fiat_connect.viewmodels"
}

dependencies {
    implementation(project(":ui"))
    api(project(":ui-models"))
    implementation(project(":data:services:store"))

    implementation(libs.lifecycle.viewmodel)
    implementation(libs.lifecycle.viewmodel.savedstate)

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(libs.junit)
    testImplementation(libs.mockk.android)
    testImplementation(libs.kotlinx.coroutines.test)
}
