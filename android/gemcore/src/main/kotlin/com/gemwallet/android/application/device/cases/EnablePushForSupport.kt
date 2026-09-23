package com.gemwallet.android.application.device.cases

import uniffi.gemstone.GemPushState

interface EnablePushForSupport {
    suspend fun enablePushForSupport(): GemPushState?
}
