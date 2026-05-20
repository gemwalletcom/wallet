package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.BuildPerpetualParams
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
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
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualType
import kotlinx.coroutines.flow.firstOrNull
import java.math.BigInteger
import kotlin.math.pow

class BuildPerpetualParamsImpl(
    private val perpetualRepository: PerpetualRepository,
    private val sessionRepository: SessionRepository,
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
        val account = sessionRepository.session().value?.wallet?.hyperliquidAccount ?: return null
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

    private suspend fun getPerpetual(perpetualId: PerpetualId): PerpetualData? =
        perpetualRepository.getPerpetual(perpetualId).firstOrNull()

    private suspend fun getPosition(perpetualId: PerpetualId): PerpetualPosition? =
        perpetualRepository.getPositionByPerpetualId(perpetualId).firstOrNull()?.position

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
