package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.BuildPerpetualParams
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.perpetual.PerpetualPositionAction
import com.gemwallet.android.domains.perpetual.PerpetualOrderFactory
import com.gemwallet.android.domains.perpetual.PerpetualTransferData
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.hyperliquidAccount
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualModifyConfirmData
import com.wallet.core.primitives.PerpetualModifyPositionType
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualType
import kotlinx.coroutines.flow.firstOrNull
import java.math.BigInteger
import kotlin.math.pow

class BuildPerpetualParamsImpl(
    private val perpetualStore: GemstonePerpetualStore,
    private val getSession: GetSession,
) : BuildPerpetualParams {

    override suspend fun open(perpetualId: PerpetualId, direction: PerpetualDirection): AmountParams.Perpetual? {
        val data = getPerpetual(perpetualId) ?: return null
        val transferData = createTransferData(data, direction, data.perpetual.maxLeverage, data.perpetual.marginType()) ?: return null
        return createAmountParams(data, PerpetualPositionAction.Open(transferData))
    }

    override suspend fun increase(perpetualId: PerpetualId): AmountParams.Perpetual? {
        val data = getPerpetual(perpetualId) ?: return null
        val position = getPosition(perpetualId) ?: return null
        val transferData = createTransferData(data, position.direction, position.leverage, position.marginType) ?: return null
        return createAmountParams(data, PerpetualPositionAction.Increase(transferData))
    }

    override suspend fun reduce(perpetualId: PerpetualId): AmountParams.Perpetual? {
        val data = getPerpetual(perpetualId) ?: return null
        val position = getPosition(perpetualId) ?: return null
        val transferData = createTransferData(data, position.direction, position.leverage, position.marginType) ?: return null
        val available = BigInteger.valueOf((position.marginAmount * 10.0.pow(HypercoreUSDC.decimals)).toLong())
        return createAmountParams(data, PerpetualPositionAction.Reduce(transferData, available, position.direction))
    }

    override suspend fun close(perpetualId: PerpetualId): ConfirmParams.PerpetualParams? {
        val data = getPerpetual(perpetualId) ?: return null
        val position = getPosition(perpetualId) ?: return null
        val assetIndex = data.perpetual.identifier.toIntOrNull() ?: return null
        val account = getSession().value?.wallet?.hyperliquidAccount ?: return null
        val confirmData = PerpetualOrderFactory.makeCloseOrder(
            assetIndex = assetIndex,
            perpetual = data.perpetual,
            position = position,
            asset = data.asset,
            baseAsset = HypercoreUSDC,
        )
        return ConfirmParams.Builder(data.asset, account)
            .perpetual(PerpetualType.Close(confirmData))
    }

    override suspend fun modify(
        perpetualId: PerpetualId,
        modifyTypes: List<PerpetualModifyPositionType>,
        takeProfitOrderId: ULong?,
        stopLossOrderId: ULong?,
    ): ConfirmParams.PerpetualParams? {
        if (modifyTypes.isEmpty()) return null
        val data = getPerpetual(perpetualId) ?: return null
        val assetIndex = data.perpetual.identifier.toIntOrNull() ?: return null
        val account = getSession().value?.wallet?.hyperliquidAccount ?: return null
        val confirmData = PerpetualModifyConfirmData(
            baseAsset = HypercoreUSDC,
            assetIndex = assetIndex,
            modifyTypes = modifyTypes,
            takeProfitOrderId = takeProfitOrderId?.toLong(),
            stopLossOrderId = stopLossOrderId?.toLong(),
        )
        return ConfirmParams.Builder(data.asset, account)
            .perpetual(PerpetualType.Modify(confirmData))
    }

    private suspend fun getPerpetual(perpetualId: PerpetualId): PerpetualData? =
        perpetualStore.observePerpetual(perpetualId).firstOrNull()

    private suspend fun getPosition(perpetualId: PerpetualId): PerpetualPosition? {
        val walletId = getSession().value?.wallet?.id ?: return null
        return perpetualStore.observePositionByPerpetualId(walletId, perpetualId).firstOrNull()?.position
    }

    private fun createTransferData(
        data: PerpetualData,
        direction: PerpetualDirection,
        leverage: UByte,
        marginType: PerpetualMarginType,
    ): PerpetualTransferData? {
        val assetIndex = data.perpetual.identifier.toIntOrNull() ?: return null
        return PerpetualTransferData(
            provider = data.perpetual.provider,
            direction = direction,
            asset = data.asset,
            baseAsset = HypercoreUSDC,
            assetIndex = assetIndex,
            price = data.perpetual.price,
            leverage = leverage,
            marginType = marginType,
        )
    }

    private fun createAmountParams(data: PerpetualData, action: PerpetualPositionAction) =
        AmountParams.Perpetual(
            assetId = data.asset.id,
            perpetualId = data.perpetual.id,
            positionAction = action,
        )

    private fun Perpetual.marginType(): PerpetualMarginType =
        if (isIsolatedOnly) PerpetualMarginType.Isolated else PerpetualMarginType.Cross
}
