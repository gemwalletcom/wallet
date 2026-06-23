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
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.features.confirm.presents.components.ConfirmErrorInfo
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.features.confirm.presents.components.FeeDetails
import com.gemwallet.android.features.confirm.presents.components.PropertyDestination
import com.gemwallet.android.features.confirm.viewmodels.ConfirmViewModel
import com.gemwallet.android.features.confirm.viewmodels.reorderWalletConnectProperties
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.perpetual.AutocloseSummaryRow
import com.gemwallet.android.ui.components.perpetual.PerpetualDetailsBottomSheet
import com.gemwallet.android.ui.components.perpetual.PerpetualDetailsSummaryItem
import com.gemwallet.android.ui.components.perpetual.title
import com.wallet.core.primitives.PerpetualType
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.NftHead
import com.gemwallet.android.ui.components.list_head.SwapListHead
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkFee
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.transaction.getTitle
import com.gemwallet.android.ui.components.list_item.walletItemIconModel
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator14
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.simulation.simulationPayloadDetailsContent
import com.gemwallet.android.ui.components.simulation.simulationPayloadFieldsContent
import com.gemwallet.android.ui.components.simulation.simulationWarningsContent
import com.gemwallet.android.ui.components.swap.SwapDetailsBottomSheet
import com.gemwallet.android.ui.components.swap.SwapDetailsSummaryItem
import com.gemwallet.android.ui.localizedDescription
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
    handleSystemBack: Boolean = false,
    viewModel: ConfirmViewModel = hiltViewModel(),
) {
    val context = LocalContext.current
    val amountModel by viewModel.amountUIModel.collectAsStateWithLifecycle()
    val txProperties by viewModel.txProperties.collectAsStateWithLifecycle()
    val feeModel by viewModel.feeUIModel.collectAsStateWithLifecycle()
    val feeValue by viewModel.feeValue.collectAsStateWithLifecycle()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val feeRates by viewModel.feeRates.collectAsStateWithLifecycle()
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

    BackHandler(handleSystemBack) {
        cancelAction()
    }

    val perpetualType by viewModel.perpetualType.collectAsStateWithLifecycle()
    Scene(
        title = confirmTitle(isWalletConnect, amountModel?.transactionType, perpetualType),
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
                    context.requestAuth(AuthRequest.Confirmation) {
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
                            walletConnectReview.headerValue?.toBigIntegerOrNull()
                                ?.let { ValueFormatter(style = ValueFormatter.Style.Full).string(it, asset) } ?: ""
                        }
                        AmountListHead(amount = title, icon = asset)
                    }
                    amountModel?.transactionType == TransactionType.Swap -> {
                        val model = requireNotNull(amountModel)
                        SwapListHead(
                            fromAsset = model.fromAsset,
                            fromValue = model.fromAmount,
                            toAsset = requireNotNull(model.toAsset),
                            toValue = requireNotNull(model.toAmount),
                            currency = model.currency,
                        )
                    }

                    amountModel?.transactionType == TransactionType.TransferNFT -> amountModel?.nftAsset?.let { NftHead(it) }

                    perpetualType != null -> {
                        val asset = amountModel?.asset?.asset
                        AmountListHead(
                            amount = asset?.symbol.orEmpty(),
                            icon = asset,
                        )
                    }

                    else -> AmountListHead(
                        amount = amountModel?.cryptoAmount ?: "",
                        equivalent = amountModel?.amountEquivalent,
                        icon = if (params is ConfirmParams.TransferParams.Withdrawal) {
                            PerpetualConfig.depositAsset
                        } else {
                            amountModel?.asset?.asset
                        },
                    )
                }
            }
            val sectionSize = displayTxProperties.size + detailElements.size
            itemsIndexed(displayTxProperties) { index, item ->
                val listPosition = ListPosition.getPosition(index, sectionSize)
                when (item) {
                    is ConfirmProperty.Destination -> PropertyDestination(
                        model = item,
                        listPosition = listPosition,
                    )
                    is ConfirmProperty.Memo -> PropertyItem(R.string.transfer_memo, item.data, listPosition = listPosition)
                    is ConfirmProperty.Network -> PropertyNetworkItem(item.data, listPosition)
                    is ConfirmProperty.Source -> PropertyItem(
                        title = { PropertyTitleText(R.string.common_wallet) },
                        data = {
                            val walletIcon = walletItemIconModel(item.walletType, item.walletChain)
                            PropertyDataText(
                                text = item.data,
                                badge = walletIcon?.let { { DataBadgeChevron(icon = it, isShowChevron = false) } },
                            )
                        },
                        listPosition = listPosition,
                    )
                }
            }
            itemsIndexed(detailElements) { index, item ->
                val listPosition = ListPosition.getPosition(displayTxProperties.size + index, sectionSize)
                ConfirmDetailElementRow(
                    item = item,
                    listPosition = listPosition,
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
                    val feeAsset = feeAssetInfo?.asset
                    val feeInfo = InfoSheetEntity.NetworkFeeInfo(
                        feeAsset?.name.orEmpty(),
                        feeAsset?.symbol.orEmpty(),
                    )
                    when (it) {
                        FeeUIModel.Calculating -> PropertyItem(
                            title = { PropertyTitleText(R.string.transfer_network_fee, info = feeInfo) },
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
                            title = { PropertyTitleText(R.string.transfer_network_fee, info = feeInfo) },
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
            feeRates = feeRates,
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
            skipPartiallyExpanded = true,
            title = stringResource(R.string.common_details),
        ) {
            LazyColumn {
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
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    when (item) {
        is ConfirmDetailElement.SwapDetails -> SwapDetailsSummaryItem(
            model = item.model,
            onClick = onClick,
            listPosition = listPosition,
        )
        is ConfirmDetailElement.PerpetualDetails -> PerpetualDetailsSummaryItem(
            model = item.model,
            onClick = onClick,
            listPosition = listPosition,
        )
        is ConfirmDetailElement.PerpetualModifyAutoclose -> AutocloseSummaryRow(
            takeProfitText = item.takeProfitText,
            stopLossText = item.stopLossText,
            listPosition = listPosition,
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

        is ConfirmDetailElement.PerpetualDetails -> PerpetualDetailsBottomSheet(
            isVisible = true,
            model = item.model,
            onDismiss = onDismiss,
        )

        is ConfirmDetailElement.PerpetualModifyAutoclose -> Unit

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
    is ConfirmError.NetworkError -> error.localizedDescription()
    is ConfirmError.SignFail -> stringResource(R.string.errors_transfer_error)
    is ConfirmError.RecipientEmpty -> "${stringResource(R.string.errors_transfer_error)}: recipient can't be empty"
    is ConfirmError.DustThreshold -> stringResource(id = R.string.errors_dust_threshold_short)
    is ConfirmError.None -> stringResource(id = R.string.transfer_confirm)
    is ConfirmError.MinimumAccountBalanceTooLow -> stringResource(R.string.transfer_minimum_account_balance, asset.symbol)
}

@Composable
private fun confirmTitle(
    isWalletConnect: Boolean,
    transactionType: TransactionType?,
    perpetualType: PerpetualType?,
): String = when {
    isWalletConnect -> stringResource(R.string.transfer_review_request)
    perpetualType != null -> perpetualType.title()
    else -> stringResource(transactionType?.getTitle() ?: R.string.transfer_title)
}
