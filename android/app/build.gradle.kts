import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

data class ReleaseKey(val store: File, val storePassword: String, val alias: String, val keyPassword: String)

val RELEASE_KEY_FILE: File = rootProject.file("../secrets/android-release.properties")

fun required(name: String): String = System.getenv(name) ?: error("$name is not set")

fun keyFromEnvironment(): ReleaseKey? {
    val store = System.getenv("MAGPIE_KEYSTORE_PATH") ?: return null
    return ReleaseKey(
        store = file(store),
        storePassword = required("MAGPIE_KEYSTORE_PASSWORD"),
        alias = required("MAGPIE_KEY_ALIAS"),
        keyPassword = required("MAGPIE_KEY_PASSWORD"),
    )
}

fun keyFromFile(): ReleaseKey? {
    if (!RELEASE_KEY_FILE.isFile) return null
    val properties = Properties().apply { RELEASE_KEY_FILE.inputStream().use(::load) }
    fun property(name: String): String = properties.getProperty(name) ?: error("$name is missing from $RELEASE_KEY_FILE")
    return ReleaseKey(
        store = RELEASE_KEY_FILE.resolveSibling(property("storeFile")),
        storePassword = property("storePassword"),
        alias = property("keyAlias"),
        keyPassword = property("keyPassword"),
    )
}

val releaseKey: ReleaseKey? = keyFromEnvironment() ?: keyFromFile()

android {
    namespace = "app.magpie.browser"
    compileSdk {
        version = release(37) { minorApiLevel = 2 }
    }

    defaultConfig {
        applicationId = "app.magpie.browser"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    signingConfigs {
        releaseKey?.let { key ->
            create("release") {
                storeFile = key.store
                storePassword = key.storePassword
                keyAlias = key.alias
                keyPassword = key.keyPassword
            }
        }
    }

    buildTypes {
        release {
            signingConfig = signingConfigs.findByName("release")
        }
    }

    splits {
        abi {
            isEnable = true
            reset()
            include("arm64-v8a", "x86_64")
            isUniversalApk = false
        }
    }

    buildFeatures {
        compose = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

dependencies {
    implementation(platform(libs.compose.bom))
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(libs.compose.ui)
    implementation(libs.compose.material3)
    implementation(libs.compose.icons)
    implementation(libs.coroutines.android)
    implementation(libs.geckoview)
    debugImplementation(libs.compose.tooling)

    testImplementation(libs.junit)
    testImplementation(libs.json)
}
