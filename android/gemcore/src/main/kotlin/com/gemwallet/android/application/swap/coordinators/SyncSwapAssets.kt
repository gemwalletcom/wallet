package com.gemwallet.android.application.swap.coordinators

import com.wallet.core.primitives.ConfigVersions

interface SyncSwapAssets {
    suspend operator fun invoke(versions: ConfigVersions)
}
