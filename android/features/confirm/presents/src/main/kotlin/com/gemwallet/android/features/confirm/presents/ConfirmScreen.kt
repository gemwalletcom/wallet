package com.gemwallet.android.features.confirm.presents

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.confirm.models.ConfirmError
import com.gemwallet.android.features.confirm.models.ConfirmProperty
import com.gemwallet.android.features.confirm.models.ConfirmState
import com.gemwallet.android.features.confirm.models.FeeUIModel
import com.gemwallet.android.features.confirm.presents.components.ConfirmErrorInfo
import com.gemwallet.android.features.confirm.presents.components.FeeDetails
import com.gemwallet.android.features.confirm.presents.components.PropertyDestination
import com.gemwallet.android.features.confirm.viewmodels.ConfirmViewModel
import com.gemwallet.android.features.confirm.viewmodels.reorderWalletConnectProperties
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.format
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.dialog.DialogBar
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.NftHead
import com.gemwallet.android.ui.components.list_head.SwapListHead
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkFee
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.transaction.getTitle
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator14
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.simulation.simulationPayloadDetailsContent
import com.gemwallet.android.ui.components.simulation.simulationPayloadFieldsContent
import com.gemwallet.android.ui.components.simulation.simulationWarningsContent
import com.gemwallet.android.ui.components.swap.SwapDetailsBottomSheet
import com.gemwallet.android.ui.components.swap.SwapDetailsSummaryItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.hasCriticalWarning
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.TransactionType

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConfirmScreen(
    params: ConfirmParams? = null,
    walletConnectSimulation: SimulationResult? = null,
    finishAction: FinishConfirmAction,
    cancelAction: CancelAction,
    onBuy: AssetIdAction,
    viewModel: ConfirmViewModel = hiltViewModel(),
) {
    val context = LocalContext.current
    val amountModel by viewModel.amountUIModel.collectAsStateWithLifecycle()
    val txProperties by viewModel.txProperties.collectAsStateWithLifecycle()
    val feeModel by viewModel.feeUIModel.collectAsStateWithLifecycle()
    val feeValue by viewModel.feeValue.collectAsStateWithLifecycle()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val allFee by viewModel.allFee.collectAsStateWithLifecycle()
    val feeAssetInfo by viewModel.feeAssetInfo.collectAsStateWithLifecycle()
    val walletConnectReview by viewModel.walletConnectReview.collectAsStateWithLifecycle()
    val detailElements by viewModel.detailElements.collectAsStateWithLifecycle()
    val isWalletConnect = params is ConfirmParams.TransferParams.Generic
    val displayTxProperties = if (isWalletConnect) txProperties.reorderWalletConnectProperties() else txProperties

    var showSelectTxSpeed by remember { mutableStateOf(false) }
    var showWalletConnectDetails by remember { mutableStateOf(false) }
    var selectedDetailElement by remember(params) { mutableStateOf<ConfirmDetailElement?>(null) }
    var isShowedBroadcastError by remember((state as? ConfirmState.BroadcastError)?.message) {
        mutableStateOf(state is ConfirmState.BroadcastError)
    }
    var isShowBottomSheetInfo by remember(state as? ConfirmState.Error) {
        mutableStateOf((state as? ConfirmState.Error)?.message is ConfirmError.InsufficientFee)
    }

    LaunchedEffect(params, walletConnectSimulation?.header?.assetId) {
        if (params != null) {
            viewModel.init(params, walletConnectSimulation)
        }
    }

    BackHandler(true) {
        cancelAction()
    }

    Scene(
        title = stringResource(
            if (isWalletConnect) {
                R.string.transfer_review_request
            } else {
                amountModel?.txType?.getTitle() ?: R.string.transfer_title
            }
        ),
        closeIcon = isWalletConnect,
        onClose = { cancelAction() },
        mainAction = {
            MainActionButton(
                title = state.buttonLabel(),
                enabled = state !is ConfirmState.Prepare
                    && state !is ConfirmState.Sending
                    && !walletConnectReview.warnings.hasCriticalWarning(),
                loading = state is ConfirmState.Sending || state is ConfirmState.Prepare || state is ConfirmState.Result,
                onClick = {
                    context.requestAuth(AuthRequest.Phrase) {
                        viewModel.send(finishAction)
                    }
                },
            )
        }
    ) { paddingValues ->
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = paddingValues.calculateBottomPadding() + paddingDefault),
        ) {
            item {
                when {
                    walletConnectReview.headerAsset != null -> {
                        val asset = requireNotNull(walletConnectReview.headerAsset)
                        val title = if (walletConnectReview.headerIsUnlimited) {
                            stringResource(R.string.simulation_header_unlimited_asset, asset.symbol)
                        } else {
                            walletConnectReview.headerValue?.toBigIntegerOrNull()?.let { asset.format(Crypto(it), dynamicPlace = true) } ?: ""
                        }
                        AmountListHead(amount = title, icon = asset)
                    }
                    amountModel?.txType == TransactionType.Swap -> {
                        val model = requireNotNull(amountModel)
                        SwapListHead(
                            fromAsset = model.fromAsset,
                            fromValue = model.fromAmount,
                            toAsset = requireNotNull(model.toAsset),
                            toValue = requireNotNull(model.toAmount),
                            currency = model.currency,
                        )
                    }

                    amountModel?.txType == TransactionType.TransferNFT -> amountModel?.nftAsset?.let { NftHead(it) }

                    else -> AmountListHead(
                        amount = amountModel?.amount ?: "",
                        equivalent = amountModel?.amountEquivalent,
                        icon = amountModel?.asset?.asset,
                    )
                }
            }
            itemsIndexed(displayTxProperties) { index, item ->
                val listPosition = ListPosition.getPosition(index, displayTxProperties.size)
                when (item) {
                    is ConfirmProperty.Destination -> PropertyDestination(item, listPosition)
                    is ConfirmProperty.Memo -> PropertyItem(R.string.transfer_memo, item.data, listPosition = listPosition)
                    is ConfirmProperty.Network -> PropertyNetworkItem(item.data, listPosition)
                    is ConfirmProperty.Source -> PropertyItem(R.string.common_wallet, item.data, listPosition = listPosition)
                }
            }
            items(
                items = detailElements,
            ) { item ->
                ConfirmDetailElementRow(
                    item = item,
                    onClick = { selectedDetailElement = item },
                )
            }
            simulationWarningsContent(walletConnectReview.warnings)
            simulationPayloadFieldsContent(
                fields = walletConnectReview.primaryPayloadFields,
                onDetailsClick = walletConnectReview.secondaryPayloadFields
                    .takeIf { it.isNotEmpty() }
                    ?.let { { showWalletConnectDetails = true } },
            )
            item {
                feeModel?.let {
                    when (it) {
                        FeeUIModel.Calculating -> PropertyItem(
                            title = { PropertyTitleText(R.string.transfer_network_fee) },
                            data = { Row(horizontalArrangement = Arrangement.End) { CircularProgressIndicator14() } },
                            listPosition = ListPosition.Single,
                        )

                        is FeeUIModel.FeeInfo -> PropertyNetworkFee(
                            it.feeAsset.name,
                            it.feeAsset.symbol,
                            it.cryptoAmount,
                            it.fiatAmount,
                            true,
                        ) { showSelectTxSpeed = true }

                        FeeUIModel.Error -> PropertyItem(
                            title = { PropertyTitleText(R.string.transfer_network_fee) },
                            data = { PropertyDataText("~") },
                            listPosition = ListPosition.Single,
                        )
                    }
                }
            }
            item {
                ConfirmErrorInfo(state, feeValue = feeValue, isShowBottomSheetInfo, onBuy)
            }
        }

        FeeDetails(
            isVisible = showSelectTxSpeed,
            currentFee = feeModel as? FeeUIModel.FeeInfo,
            fee = allFee,
            feeAssetInfo = feeAssetInfo,
            onSelect = {
                showSelectTxSpeed = false
                viewModel.changeFeePriority(it)
            },
            onCancel = { showSelectTxSpeed = false },
        )

        ModalBottomSheet(
            isVisible = showWalletConnectDetails,
            onDismissRequest = { showWalletConnectDetails = false },
            sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
            dragHandle = {
                DialogBar(
                    onDismissRequest = { showWalletConnectDetails = false },
                )
            },
        ) {
            LazyColumn {
                item {
                    SubheaderItem(R.string.common_details)
                }
                simulationPayloadDetailsContent(
                    primaryFields = walletConnectReview.primaryPayloadFields,
                    secondaryFields = walletConnectReview.secondaryPayloadFields,
                )
            }
        }

        ConfirmDetailElementBottomSheet(
            item = selectedDetailElement,
            onDismiss = { selectedDetailElement = null },
        )
    }

    if (isShowedBroadcastError) {
        AlertDialog(
            onDismissRequest = { isShowedBroadcastError = false },
            confirmButton = {
                Button({ isShowedBroadcastError = false }) { Text(stringResource(R.string.common_done)) }
            },
            title = {
                Text(stringResource(R.string.errors_transfer_error))
            },
            text = {
                Text((state as? ConfirmState.BroadcastError)?.message?.toLabel() ?: "Unknown error")
            }
        )
    }
}

@Composable
private fun ConfirmDetailElementRow(
    item: ConfirmDetailElement,
    onClick: () -> Unit,
) {
    when (item) {
        is ConfirmDetailElement.SwapDetails -> SwapDetailsSummaryItem(
            model = item.model,
            onClick = onClick,
        )
    }
}

@Composable
private fun ConfirmDetailElementBottomSheet(
    item: ConfirmDetailElement?,
    onDismiss: () -> Unit,
) {
    when (item) {
        is ConfirmDetailElement.SwapDetails -> SwapDetailsBottomSheet(
            isVisible = true,
            isLoading = false,
            model = item.model,
            onDismiss = onDismiss,
            showProviderSectionHeader = true,
        )

        null -> Unit
    }
}

@Composable
fun ConfirmState.buttonLabel(): String {
    return when (this) {
        is ConfirmState.BroadcastError,
        is ConfirmState.Error -> stringResource(R.string.common_try_again)
        is ConfirmState.FatalError -> message
        ConfirmState.Prepare,
        ConfirmState.Ready,
        is ConfirmState.Result,
        ConfirmState.Sending -> stringResource(id = R.string.transfer_confirm)
    }
}

@Composable
fun ConfirmError.toLabel() = when (this) {
    is ConfirmError.Init,
    is ConfirmError.TransactionIncorrect,
    is ConfirmError.PreloadError -> "${stringResource(R.string.confirm_fee_error)}: ${stringResource(R.string.errors_unable_estimate_network_fee)}"
    is ConfirmError.InsufficientBalance -> stringResource(R.string.transfer_insufficient_balance, chainTitle)
    is ConfirmError.InsufficientFee -> stringResource(R.string.transfer_insufficient_network_fee_balance, chain.asset().name)
    is ConfirmError.BroadcastError -> "${stringResource(R.string.errors_transfer_error)}: ${this.details}"
    is ConfirmError.SignFail -> stringResource(R.string.errors_transfer_error)
    is ConfirmError.RecipientEmpty -> "${stringResource(R.string.errors_transfer_error)}: recipient can't empty"
    is ConfirmError.DustThreshold -> stringResource(id = R.string.errors_dust_threshold, chainTitle)
    is ConfirmError.None -> stringResource(id = R.string.transfer_confirm)
    is ConfirmError.MinimumAccountBalanceTooLow -> stringResource(R.string.transfer_minimum_account_balance, asset.symbol)
}
