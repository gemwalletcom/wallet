package com.gemwallet.android.features.perpetual.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.coordinators.BuildPerpetualParams
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseEstimator
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseField
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseModifyBuilder
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseValidator
import com.gemwallet.android.ext.PerpetualFormatter
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.NumericFormatter
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModel
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModelFactory
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.TpslType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.text.DecimalFormatSymbols
import java.util.Locale
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AutocloseViewModel @Inject constructor(
    private val perpetualRepository: PerpetualRepository,
    private val buildPerpetualParams: BuildPerpetualParams,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val assetId: AssetId = savedStateHandle.requireAssetId()

    private val numericFormatter = NumericFormatter()

    val position: StateFlow<PerpetualPositionData?> = perpetualRepository.getPerpetualByAssetId(assetId)
        .distinctUntilChanged()
        .flatMapLatest { data ->
            data?.let { perpetualRepository.getPositionByPerpetualId(it.perpetual.id) } ?: flowOf(null)
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val _confirmRequests = MutableSharedFlow<ConfirmParams.PerpetualParams>(extraBufferCapacity = 1)
    val confirmRequests: SharedFlow<ConfirmParams.PerpetualParams> = _confirmRequests

    private val userTakeProfitText = MutableStateFlow<String?>(null)
    private val userStopLossText = MutableStateFlow<String?>(null)

    val takeProfitText: StateFlow<String> = combine(userTakeProfitText, position) { user, pos ->
        user ?: initialText(pos, TpslType.TakeProfit)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val stopLossText: StateFlow<String> = combine(userStopLossText, position) { user, pos ->
        user ?: initialText(pos, TpslType.StopLoss)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val submitAttempted = MutableStateFlow(false)

    val uiModel: StateFlow<AutocloseUIModel?> = combine(
        position,
        takeProfitText,
        stopLossText,
        submitAttempted,
    ) { position, takeProfit, stopLoss, attempted ->
        position?.let { buildUiModel(it, takeProfit, stopLoss, attempted) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun onTakeProfitChanged(text: String) {
        submitAttempted.value = false
        userTakeProfitText.value = text.filterNumeric()
    }

    fun onStopLossChanged(text: String) {
        submitAttempted.value = false
        userStopLossText.value = text.filterNumeric()
    }

    fun onPercentSelected(type: TpslType, percent: Int) {
        submitAttempted.value = false
        val position = position.value ?: return
        val estimator = estimator(position)
        val target = estimator.targetPriceFromRoe(percent, type)
        val formatted = PerpetualFormatter.formatInputPrice(
            provider = position.perpetual.provider,
            price = target,
            decimals = position.asset.decimals,
        )
        when (type) {
            TpslType.TakeProfit -> userTakeProfitText.value = formatted
            TpslType.StopLoss -> userStopLossText.value = formatted
        }
    }

    fun onConfirm() {
        submitAttempted.value = true
        val position = position.value ?: return
        val perpetualId = position.perpetual.id
        val assetIndex = position.perpetual.identifier.toIntOrNull() ?: return
        val takeProfitField = autocloseField(position, TpslType.TakeProfit, takeProfitText.value)
        val stopLossField = autocloseField(position, TpslType.StopLoss, stopLossText.value)
        val builder = AutocloseModifyBuilder(position.position.direction)
        if (!builder.canBuild(takeProfitField, stopLossField)) return
        val modifyTypes = builder.build(assetIndex, takeProfitField, stopLossField)
        viewModelScope.launch {
            buildPerpetualParams.modify(
                perpetualId = perpetualId,
                modifyTypes = modifyTypes,
                takeProfitOrderId = takeProfitField.orderId,
                stopLossOrderId = stopLossField.orderId,
            )?.let { _confirmRequests.tryEmit(it) }
        }
    }

    private fun buildUiModel(
        position: PerpetualPositionData,
        takeProfitText: String,
        stopLossText: String,
        submitAttempted: Boolean,
    ): AutocloseUIModel {
        val takeProfit = autocloseField(position, TpslType.TakeProfit, takeProfitText)
        val stopLoss = autocloseField(position, TpslType.StopLoss, stopLossText)
        val builder = AutocloseModifyBuilder(position.position.direction)
        val confirmEnabled = if (submitAttempted) {
            builder.canBuild(takeProfit, stopLoss)
        } else {
            takeProfit.hasPendingChange || stopLoss.hasPendingChange
        }
        return AutocloseUIModelFactory.create(
            position = position,
            takeProfit = takeProfit,
            stopLoss = stopLoss,
            confirmEnabled = confirmEnabled,
            showErrors = submitAttempted,
        )
    }

    private fun autocloseField(
        position: PerpetualPositionData,
        type: TpslType,
        text: String,
    ): AutocloseField {
        val price = numericFormatter.double(text)
        val original = when (type) {
            TpslType.TakeProfit -> position.position.takeProfit
            TpslType.StopLoss -> position.position.stopLoss
        }
        val validator = AutocloseValidator(
            type = type,
            direction = position.position.direction,
            marketPrice = position.perpetual.price,
        )
        return AutocloseField(
            type = type,
            price = price,
            originalPrice = original?.price,
            formattedPrice = price?.let {
                PerpetualFormatter.formatPrice(position.perpetual.provider, it, position.asset.decimals)
            },
            error = validator.error(price),
            orderId = original?.order_id?.toULongOrNull(),
        )
    }

    private fun estimator(position: PerpetualPositionData) = AutocloseEstimator(
        entryPrice = position.position.entryPrice,
        positionSize = position.position.size,
        direction = position.position.direction,
        leverage = position.position.leverage,
    )

    private fun initialText(position: PerpetualPositionData?, type: TpslType): String {
        val trigger = position?.let {
            when (type) {
                TpslType.TakeProfit -> it.position.takeProfit
                TpslType.StopLoss -> it.position.stopLoss
            }
        } ?: return ""
        return PerpetualFormatter.formatInputPrice(
            provider = position.perpetual.provider,
            price = trigger.price,
            decimals = position.asset.decimals,
        )
    }

    private fun String.filterNumeric(locale: Locale = Locale.getDefault()): String {
        val separator = DecimalFormatSymbols.getInstance(locale).decimalSeparator
        return filter { it.isDigit() || it == separator || it == '.' }
    }
}
