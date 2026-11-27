import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.tasks.KotlinJvmCompile

plugins {
    id("com.android.application")
    alias(libs.plugins.compose.compiler)
    id("com.google.dagger.hilt.android")
    id("kotlinx-serialization")
    id("com.google.devtools.ksp")
}

apply(from = "$rootDir/gradle/channels.gradle.kts")

@Suppress("UNCHECKED_CAST")
val gemChannels = extra["gemChannels"] as Map<String, Map<String, Any?>>
val gemChannel = extra["gemChannel"] as String

gradle.startParameter.projectProperties["channel"]?.takeIf { it.isNotBlank() }?.let { explicit ->
    val requestedChannels = gradle.startParameter.taskNames
        .asSequence()
        .flatMap { task ->
            gemChannels.keys.asSequence().filter { channel ->
                task.contains("assemble$channel", ignoreCase = true) || task.contains("bundle$channel", ignoreCase = true)
            }
        }
        .toSet()
        .filter { it != explicit }
    require(requestedChannels.isEmpty()) {
        "Channel mismatch: -Pchannel=$explicit but requested task(s) for $requestedChannels. Build one channel per invocation."
    }
}

if (file("google-services.json").isFile) {
    apply(plugin = "com.google.gms.google-services")

    tasks.configureEach {
        if (name.startsWith("processFdroid") && name.endsWith("GoogleServices")) {
            enabled = false
        }
    }
}

repositories {
    google()
    mavenCentral()
}

dependencyLocking {
    lockAllConfigurations()
    lockMode = LockMode.STRICT
}

configurations.matching { it.name == "androidApis" }.configureEach {
    resolutionStrategy.deactivateDependencyLocking()
}

android {
    namespace = "com.gemwallet.android"
    compileSdk = 37
    ndkVersion = libs.versions.androidNdk.get()

    val channelDimension = "channel"
    flavorDimensions.add(channelDimension)

    defaultConfig {
        applicationId = "com.gemwallet.android"
        minSdk = 28
        targetSdk = 37
        versionCode = 918
        versionName = "2.114.105"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables {
            useSupportLibrary = true
        }

        splits {
            abi {
                isEnable = false
                include("arm64-v8a", "armeabi-v7a")
                isUniversalApk = false
            }
        }
    }
    productFlavors {
        gemChannels.forEach { (name, channel) ->
            create(name) {
                dimension = channelDimension
                isDefault = channel["isDefault"] == true
                (channel["proguardFiles"] as? List<*>)
                    ?.filterIsInstance<String>()
                    ?.forEach(::proguardFile)
                ndk {
                    @Suppress("UNCHECKED_CAST")
                    abiFilters.addAll(channel.getValue("abis") as List<String>)
                }
                val updateUrl = channel.getValue("updateUrl") as String
                val updateUrlEnv = channel["updateUrlEnv"] as String?
                val resolvedUpdateUrl = (updateUrlEnv?.let { providers.environmentVariable(it).orNull } ?: updateUrl)
                    .removeSurrounding("\"")
                    .replace("\\", "\\\\")
                    .replace("\"", "\\\"")
                buildConfigField("String", "UPDATE_URL", "\"$resolvedUpdateUrl\"")
            }
        }
    }
    buildTypes {
        getByName("debug") {
            isMinifyEnabled = false
            isShrinkResources = false
            isDebuggable = true

            if (System.getenv("UNIT_TESTS") == "true") {
                ndk {
                    abiFilters.remove("arm64-v8a")
                    abiFilters.remove("armeabi-v7a")
                    abiFilters.add("x86_64")
                }

                splits {
                    abi {
                        reset()
                        isEnable = false
                        include("x86_64")
                        isUniversalApk = false
                    }
                }
            }

            buildConfigField("String", "TEST_PHRASE", "${System.getenv("TEST_PHRASE")}")
        }

        getByName("release") {
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
            isMinifyEnabled = true
            isShrinkResources = true
            isDebuggable = false
        }

    }

    packaging {
        resources {
            excludes += "META-INF/*"
            excludes += "META-INF/DEPENDENCIES"
            excludes += "/META-INF/LICENSE-notice.md"
            excludes += "/META-INF/LICENSE.md"
            excludes += "META-INF/versions/9/OSGI-INF/MANIFEST.MF"
            excludes += "org/bouncycastle/pqc/legacy/picnic/*.properties"
            excludes += "org/bouncycastle/x509/CertPathReviewerMessages*.properties"
            excludes += "org/spongycastle/x509/CertPathReviewerMessages*.properties"
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
    buildFeatures {
        compose = true
        buildConfig = true
    }
    dependenciesInfo {
        val enabled = gemChannel != "fdroid"
        includeInApk = enabled
        includeInBundle = enabled
    }
    packaging {
        jniLibs {
            useLegacyPackaging = providers.gradleProperty("useLegacyPackaging").getOrElse("false").toBoolean()
        }
    }

    androidResources {
        generateLocaleConfig = true
        localeFilters += listOf("en") + rootProject.file("ui/src/main/res").list().orEmpty()
            .filter { it.startsWith("values-") && rootProject.file("ui/src/main/res/$it/strings.xml").exists() }
            .map { it.removePrefix("values-") }
            .sorted()
    }

    lint {
        disable += "Instantiatable"
        checkGeneratedSources = true
        checkDependencies = false
    }
}

dependencies {
    implementation(project(":ui"))
    implementation(project(":data:services:native-provider"))
    implementation(project(":data:services:gemstone"))
    implementation(project(":data:services:store"))
    implementation(project(":data:coordinators"))

    // Features
    implementation(project(":features:transactions:presents"))
    implementation(project(":features:assets:presents"))
    implementation(project(":features:assets:viewmodels"))
    implementation(project(":features:qr_scanner:presents"))
    implementation(project(":features:fiat_connect:presents"))
    implementation(project(":features:transfer:presents"))
    implementation(project(":features:transfer:viewmodels"))
    implementation(project(":features:swap:presents"))
    implementation(project(":features:onboarding:presents"))
    implementation(project(":features:stake:presents"))
    implementation(project(":features:settings:presents"))
    implementation(project(":features:settings:viewmodels"))
    implementation(project(":features:contacts:presents"))
    implementation(project(":features:price_alerts:presents"))
    implementation(project(":features:in_app_notifications:presents"))
    implementation(project(":features:support:presents"))
    implementation(project(":features:nft:presents"))
    implementation(project(":features:wallet_connector:presents"))
    implementation(project(":features:wallet_connector:viewmodels"))
    implementation(project(":features:wallet:presents"))
    implementation(project(":features:wallet:viewmodels"))
    implementation(project(":features:perpetuals:presents"))
    implementation(project(":features:rewards:presents"))

    implementation(libs.ktx.core)
    implementation(libs.androidx.core.splashscreen)
    implementation(libs.okhttp)
    implementation(libs.coil.network)

    implementation(libs.tink)

    implementation(libs.hilt.lifecycle.viewmodel.compose)
    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    implementation(libs.lifecycle.process)
    implementation(libs.lifecycle.runtime.compose)
    implementation(libs.datastore)

    implementation(libs.navigation3.runtime)
    implementation(libs.navigation3.ui)

    // Legacy encrypted preferences migration
    implementation(libs.androidx.security.crypto)
    // Auth
    implementation(libs.androidx.biometric)

    gemChannels.forEach { (name, channel) ->
        val configuration = "${name}Implementation"
        listOf("push", "review").forEach { key ->
            val module = channel[key] as String
            if (rootProject.findProject(module) != null) {
                add(configuration, project(module))
            }
        }
    }

    // Tests


    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(libs.mockk)
    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.uiautomator)
    androidTestImplementation(libs.androidx.junit.runner)
}
