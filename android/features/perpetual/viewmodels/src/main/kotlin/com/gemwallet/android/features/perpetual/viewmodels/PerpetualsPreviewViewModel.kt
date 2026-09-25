package com.gemwallet.android.features.perpetual.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.preferences.cases.ObservablePreferences
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.PerpetualPositionsQuery
import com.gemwallet.android.data.services.store.queries.PerpetualWalletBalanceQuery
import com.gemwallet.android.domains.balance.hiddenWhen
import com.gemwallet.android.domains.perpetual.aggregates.positionAggregates
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualPositionRowUIModel
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
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
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.perpetualBalanceTotal
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PerpetualsPreviewViewModel @Inject constructor(
    preferences: ObservablePreferences,
    getSession: GetSession,
    perpetualPositionsQuery: PerpetualPositionsQuery,
    perpetualWalletBalanceQuery: PerpetualWalletBalanceQuery,
    @param:IoDispatcher ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val balance = getSession()
        .filterNotNull()
        .distinctUntilChangedBy { it.wallet.id }
        .flatMapLatest { perpetualWalletBalanceQuery(it.wallet.id, HypercoreUSDC.id) }
        .map { it?.balance }
        .distinctUntilChanged()

    private val positionAggregates = getSession()
        .filterNotNull()
        .flatMapLatest { perpetualPositionsQuery(it.wallet.id) }
        .map { it.positionAggregates() }
        .flowOn(ioDispatcher)

    val tradeListItem = combine(balance, preferences.isHideBalances()) { balance, hideBalance ->
        ListItemModel(
            title = context.getString(R.string.perpetuals_trade),
            subtitle = perpetualBalanceTotal(balance?.toGem()).text().hiddenWhen(hideBalance),
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ListItemModel(title = context.getString(R.string.perpetuals_trade)))

    val positions = combine(positionAggregates, preferences.isHideBalances()) { positions, hideBalance ->
        positions.map { PerpetualPositionRowUIModel(it.asset, it.row, hideBalance) }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
}
