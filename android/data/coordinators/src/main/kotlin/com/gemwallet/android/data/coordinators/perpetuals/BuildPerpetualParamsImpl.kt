package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.BuildPerpetualParams
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualPosition
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemPerpetualDetailsServiceInterface
import uniffi.gemstone.GemPerpetualPositionKind

class BuildPerpetualParamsImpl(
    private val perpetualStore: GemstonePerpetualStore,
    private val getSession: GetSession,
    private val service: GemPerpetualDetailsServiceInterface,
) : BuildPerpetualParams {

    override suspend fun position(perpetualId: PerpetualId, kind: GemPerpetualPositionKind): AmountParams.Perpetual? {
        val data = getPerpetual(perpetualId) ?: return null
        val action = service.positionAction(data.perpetual.toJson(), data.asset.toGem(), getPosition(perpetualId)?.toJson(), kind)
        return AmountParams.Perpetual(assetId = data.asset.id, perpetualId = data.perpetual.id, positionAction = action)
    }

    override suspend fun close(perpetualId: PerpetualId): GemTransferData? {
        val data = getPerpetual(perpetualId) ?: return null
        return service.closeTransfer(data.perpetual.toJson(), data.asset.toGem(), getPosition(perpetualId)?.toJson())
    }

    private suspend fun getPerpetual(perpetualId: PerpetualId): PerpetualData? =
        perpetualStore.observePerpetual(perpetualId).firstOrNull()

    private suspend fun getPosition(perpetualId: PerpetualId): PerpetualPosition? {
        val walletId = getSession().value?.wallet?.id ?: return null
        return perpetualStore.observePositionByPerpetualId(walletId, perpetualId).firstOrNull()?.position
    }
}
