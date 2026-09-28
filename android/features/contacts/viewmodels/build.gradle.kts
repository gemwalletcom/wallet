plugins {
    alias(libs.plugins.android.library)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.contacts.viewmodels"
}

dependencies {
    api(project(":ui-models"))
    implementation(project(":ui"))
    implementation(project(":data:services:store"))

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    implementation(libs.lifecycle.viewmodel)
    implementation(libs.lifecycle.viewmodel.savedstate)

    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)
    testImplementation(libs.mockk.android)
    testImplementation(testFixtures(project(":gemcore")))
}
