package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.ConfigVersions

interface SyncFiatAssets {
    suspend operator fun invoke(versions: ConfigVersions)
}
