package com.gemwallet.android.features.perpetual.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositionByAsset
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.perpetual.listItem
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.TpslType
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemAutocloseSession
import uniffi.gemstone.GemAutocloseViewState
import uniffi.gemstone.autocloseSession
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AutocloseViewModel @Inject constructor(
    private val getPositionByAsset: GetPerpetualPositionByAsset,
    private val getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val assetId: AssetId = savedStateHandle.requireAssetId()

    val position: StateFlow<PerpetualPositionData?> = getSession()
        .filterNotNull()
        .flatMapLatest { session -> getPositionByAsset(session.wallet.id, assetId) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val _confirmRequests = MutableSharedFlow<ConfirmTransferInput>(extraBufferCapacity = 1)
    val confirmRequests: SharedFlow<ConfirmTransferInput> = _confirmRequests

    private val _errors = MutableSharedFlow<String>(extraBufferCapacity = 1)
    val errors: SharedFlow<String> = _errors

    private val userTakeProfitText = MutableStateFlow<String?>(null)
    private val userStopLossText = MutableStateFlow<String?>(null)

    val takeProfitText: StateFlow<String> = combine(userTakeProfitText, position) { user, pos ->
        user ?: initialText(pos, TpslType.TakeProfit)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val stopLossText: StateFlow<String> = combine(userStopLossText, position) { user, pos ->
        user ?: initialText(pos, TpslType.StopLoss)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val submitAttempted = MutableStateFlow(false)

    val viewState: StateFlow<GemAutocloseViewState?> = combine(
        position,
        takeProfitText,
        stopLossText,
        submitAttempted,
    ) { position, takeProfit, stopLoss, attempted ->
        position?.let { session(it, takeProfit, stopLoss).let { session -> if (attempted) session.onSubmitAttempt() else session }.viewState() }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val positionListItem: StateFlow<ListItemModel?> = viewState.map { it?.positionRow?.listItem(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun onTakeProfitChanged(text: String) {
        submitAttempted.value = false
        userTakeProfitText.value = numberFormat().sanitize(text, null, null)
    }

    fun onStopLossChanged(text: String) {
        submitAttempted.value = false
        userStopLossText.value = numberFormat().sanitize(text, null, null)
    }

    fun onPercentSelected(type: TpslType, percent: Int) {
        submitAttempted.value = false
        val position = position.value ?: return
        val text = session(position, takeProfitText.value, stopLossText.value)
            .onPercentSelected(type.toGem(), percent)
            .inputText(type.toGem(), numberFormat().decimalSeparator.toString())
        when (type) {
            TpslType.TakeProfit -> userTakeProfitText.value = text
            TpslType.StopLoss -> userStopLossText.value = text
        }
    }

    fun onConfirm() {
        submitAttempted.value = true
        val position = position.value ?: return
        val session = session(position, takeProfitText.value, stopLossText.value).onSubmitAttempt()
        if (!session.viewState().confirmEnabled) return
        val transfer = runCatching { session.modify.transfer(position.perpetual.provider.toGem(), position.asset.toGem()) }.getOrElse { error ->
            _errors.tryEmit(error.errorText().text(context))
            return
        }
        _confirmRequests.tryEmit(ConfirmTransferInput(transfer))
    }

    private fun session(position: PerpetualPositionData, takeProfitText: String, stopLossText: String): GemAutocloseSession = autocloseSession(position.perpetual.toGem(), position.asset.toGem(), position.position.toGem())
        .onPrice(TpslType.TakeProfit.toGem(), takeProfitText.parseInputNumberOrNull()?.toDouble())
        .onPrice(TpslType.StopLoss.toGem(), stopLossText.parseInputNumberOrNull()?.toDouble())

    private fun initialText(position: PerpetualPositionData?, type: TpslType): String {
        position ?: return ""
        return autocloseSession(position.perpetual.toGem(), position.asset.toGem(), position.position.toGem())
            .inputText(type.toGem(), numberFormat().decimalSeparator.toString())
            .orEmpty()
    }
}
