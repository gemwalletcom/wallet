plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.settings.viewmodels"
}

dependencies {
    implementation(project(":gemcore"))
    implementation(project(":data:services:store"))
    api(project(":ui-models"))
    implementation(project(":ui"))

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    implementation(libs.lifecycle.viewmodel)

    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk.android)
    testImplementation(testFixtures(project(":gemcore")))
}
