package com.gemwallet.android.features.perpetuals.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.preferences.cases.ObservablePreferences
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.PerpetualPositionsQuery
import com.gemwallet.android.data.services.store.queries.PerpetualWalletBalanceQuery
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.perpetuals.viewmodels.models.PerpetualPositionRowUIModel
import com.gemwallet.android.features.perpetuals.viewmodels.models.PerpetualTradeRowUIModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemPerpetualPreview
import uniffi.gemstone.perpetualPositionRows
import uniffi.gemstone.perpetualPreview
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PerpetualsPreviewViewModel @Inject constructor(
    preferences: ObservablePreferences,
    getSession: GetSession,
    perpetualPositionsQuery: PerpetualPositionsQuery,
    perpetualWalletBalanceQuery: PerpetualWalletBalanceQuery,
    @param:IoDispatcher ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val balance = getSession()
        .filterNotNull()
        .distinctUntilChangedBy { it.wallet.id }
        .flatMapLatest { perpetualWalletBalanceQuery(it.wallet.id, HypercoreUSDC.id) }
        .map { it?.balance }
        .distinctUntilChanged()

    private val positionData = getSession()
        .filterNotNull()
        .flatMapLatest { perpetualPositionsQuery(it.wallet.id) }
        .flowOn(ioDispatcher)
        .shareIn(viewModelScope, SharingStarted.Eagerly, replay = 1)

    val tradeRow = combine(positionData, balance, preferences.isHideBalances()) { positions, balance, hideBalance ->
        when (val preview = perpetualPreview(positions.map { it.position.id }, balance?.toGem())) {
            is GemPerpetualPreview.Trade -> PerpetualTradeRowUIModel(preview.balance, hideBalance)
            GemPerpetualPreview.Positions -> null
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val positions = combine(positionData, preferences.isHideBalances()) { positions, hideBalance ->
        positions.zip(perpetualPositionRows(positions.map { it.toGem() })) { data, row -> PerpetualPositionRowUIModel(data.asset, row.row, hideBalance) }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
}
