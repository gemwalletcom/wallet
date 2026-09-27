package com.gemwallet.android.features.stake.viewmodels

import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.toAmountParams
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.wallet.core.primitives.Delegation
import uniffi.gemstone.GemDelegationDestination

internal fun GemDelegationDestination.open(delegation: Delegation, onOpenDetail: (String, String) -> Unit, onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction) = when (this) {
    GemDelegationDestination.Details -> onOpenDetail(delegation.validator.id, delegation.base.delegationId)
    is GemDelegationDestination.Confirm -> onConfirm(ConfirmTransferInput(transfer))
    is GemDelegationDestination.Amount -> onAmount(input.toAmountParams(asset.toPrimitives().id))
}
