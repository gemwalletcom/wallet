package com.gemwallet.android.data.services.gemstone.stores

import android.os.Looper
import android.util.Log

internal fun probeMainThread(name: String) {
    if (Looper.myLooper() == Looper.getMainLooper()) {
        Log.e("MainThreadProbe", "SYNC STORE CALLBACK ON MAIN: $name", Throwable())
    }
}
