package com.gemwallet.android.features.stake.viewmodels.delegation.models

import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemDelegationDetails

class DelegationUIState(val rows: List<DelegationRowUIModel>, val details: GemDelegationDetails, val actions: List<DelegationActionUIModel>, val asset: Asset)
