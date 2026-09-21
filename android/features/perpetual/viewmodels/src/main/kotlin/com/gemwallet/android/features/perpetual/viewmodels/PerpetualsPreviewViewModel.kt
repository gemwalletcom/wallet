package com.gemwallet.android.features.perpetual.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.config.showPerpetuals
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualPositionRowUIModel
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.perpetual.listItem
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.perpetualBalanceHeader
import javax.inject.Inject

@HiltViewModel
class PerpetualsPreviewViewModel @Inject constructor(userConfig: UserConfig, getSession: GetSession, getPositions: GetPerpetualPositions, getBalance: GetPerpetualBalance, @param:ApplicationContext private val context: Context) :
    ViewModel() {

    val tradeListItem = combine(getBalance.getBalance(), getSession()) { balance, session ->
        ListItemModel(
            title = context.getString(R.string.perpetuals_trade),
            subtitle = perpetualBalanceHeader(balance?.toGem(), (session?.wallet?.type ?: WalletType.View).toGem()).total.text(),
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ListItemModel(title = context.getString(R.string.perpetuals_trade)))

    val showPerpetuals = userConfig.showPerpetuals(getSession())
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val positions = getPositions.getPerpetualPositions()
        .map { positions -> positions.map { PerpetualPositionRowUIModel(it.asset, it.listItem(context)) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
}
