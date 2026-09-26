package com.gemwallet.android.features.perpetuals.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.PerpetualPositionsQuery
import com.gemwallet.android.data.services.store.queries.PerpetualQuery
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.ui.R
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
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.updateAndGet
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemAutocloseSession
import uniffi.gemstone.GemAutocloseViewState
import uniffi.gemstone.autocloseSession
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AutocloseViewModel @Inject constructor(
    private val perpetualQuery: PerpetualQuery,
    private val perpetualPositionsQuery: PerpetualPositionsQuery,
    private val getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val assetId: AssetId = savedStateHandle.requireAssetId()

    val position: StateFlow<PerpetualPositionData?> = getSession()
        .filterNotNull()
        .flatMapLatest { session ->
            perpetualQuery(assetId)
                .distinctUntilChanged()
                .flatMapLatest { data -> data?.let { perpetualPositionsQuery(session.wallet.id, it.perpetual.id) } ?: flowOf(null) }
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val _confirmRequests = MutableSharedFlow<ConfirmTransferInput>(extraBufferCapacity = 1)
    val confirmRequests: SharedFlow<ConfirmTransferInput> = _confirmRequests

    private val _errors = MutableSharedFlow<String>(extraBufferCapacity = 1)
    val errors: SharedFlow<String> = _errors

    private val session = MutableStateFlow<GemAutocloseSession?>(null)

    val viewState: StateFlow<GemAutocloseViewState?> = combine(position, session) { position, session ->
        (session ?: position?.let(::newSession))?.viewState()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val positionRow: StateFlow<GemAssetItemRow?> = viewState.map { it?.positionRow?.row }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun onTakeProfitChanged(text: String) {
        updateSession { it.onInput(TpslType.TakeProfit.toGem(), numberFormat().sanitize(text, null, null)) }
    }

    fun onStopLossChanged(text: String) {
        updateSession { it.onInput(TpslType.StopLoss.toGem(), numberFormat().sanitize(text, null, null)) }
    }

    fun onPercentSelected(type: TpslType, percent: Int) {
        updateSession { it.onPercentSelected(type.toGem(), percent) }
    }

    fun onConfirm() {
        val position = position.value ?: return
        val session = updateSession { it.onSubmitAttempt() } ?: return
        if (!session.viewState().confirmEnabled) return
        val transfer = runCatching { session.modify.transfer(position.perpetual.provider.toGem(), position.asset.toGem()) }.getOrElse { error ->
            _errors.tryEmit(error.errorText().text(context))
            return
        }
        _confirmRequests.tryEmit(ConfirmTransferInput(transfer))
    }

    private fun updateSession(transform: (GemAutocloseSession) -> GemAutocloseSession): GemAutocloseSession? {
        val position = position.value ?: return null
        return session.updateAndGet { transform(it ?: newSession(position)) }
    }

    private fun newSession(position: PerpetualPositionData): GemAutocloseSession = autocloseSession(position.perpetual.toGem(), position.asset.toGem(), position.position.toGem(), numberFormat())
}
