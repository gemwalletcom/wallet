package com.gemwallet.android.application.perpetual.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetPerpetualPositionByAsset {
    operator fun invoke(walletId: WalletId, assetId: AssetId): Flow<PerpetualPositionData?>
}
