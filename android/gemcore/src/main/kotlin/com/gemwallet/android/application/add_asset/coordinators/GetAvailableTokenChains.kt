package com.gemwallet.android.application.add_asset.coordinators

import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow

interface GetAvailableTokenChains {
    operator fun invoke(): Flow<List<Chain>?>
}
