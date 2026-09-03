package com.gemwallet.android.features.confirm.presents

import androidx.activity.compose.BackHandler
import androidx.annotation.StringRes
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
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
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.asset.title
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.boldMarkdown
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.ext.toGemNetworkError
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.GemNetworkError
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemSignerError
import java.math.BigInteger
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.features.confirm.presents.components.ConfirmErrorInfo
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.features.confirm.presents.components.FeeDetails
import com.gemwallet.android.features.confirm.presents.components.PropertyDestination
import com.gemwallet.android.features.confirm.viewmodels.ConfirmViewModel
import com.gemwallet.android.features.confirm.viewmodels.reorderRequestProperties
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.domains.confirm.applicationMetadata
import com.gemwallet.android.domains.confirm.asset
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemTransactionHeaderKind
import uniffi.gemstone.GemTransactionInputType
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.perpetual.AutocloseSummaryRow
import com.gemwallet.android.ui.components.perpetual.PerpetualDetailsBottomSheet
import com.gemwallet.android.ui.components.perpetual.PerpetualDetailsSummaryItem
import com.gemwallet.android.ui.components.perpetual.title
import com.wallet.core.primitives.PerpetualType
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ApplicationMetadataSource
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.image.walletImageModel
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
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.features.confirm.presents.components.confirmBalanceChangesContent
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.TransactionType

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConfirmScreen(
    input: GemTransferData? = null,
    simulationResult: SimulationResult? = null,
    finishAction: FinishConfirmAction,
    cancelAction: CancelAction,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    handleSystemBack: Boolean = false,
    viewModel: ConfirmViewModel = hiltViewModel(),
) {
    val context = LocalContext.current
    val amountModel by viewModel.amountUIModel.collectAsStateWithLifecycle()
    val transactionProperties by viewModel.transactionProperties.collectAsStateWithLifecycle()
    val feeModel by viewModel.feeUIModel.collectAsStateWithLifecycle()
    val feeValue by viewModel.feeValue.collectAsStateWithLifecycle()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val feeRates by viewModel.feeRates.collectAsStateWithLifecycle()
    val feeAssets by viewModel.feeAssets.collectAsStateWithLifecycle()
    val feeAsset by viewModel.feeAsset.collectAsStateWithLifecycle()
    val feeSelection by viewModel.feeSelection.collectAsStateWithLifecycle()
    val simulation by viewModel.simulation.collectAsStateWithLifecycle()
    val detailElements by viewModel.detailElements.collectAsStateWithLifecycle()
    val payloadAddressNames by viewModel.payloadAddressNames.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()
    val applicationMetadata = input?.inputType?.applicationMetadata
    val isExternalRequest = applicationMetadata != null
    val isPayment = applicationMetadata?.source == ApplicationMetadataSource.Payment
    val displayTransactionProperties = if (isExternalRequest) transactionProperties.reorderRequestProperties() else transactionProperties

    var showSelectTxSpeed by remember { mutableStateOf(false) }
    var showSimulationDetails by remember { mutableStateOf(false) }
    var selectedDetailElement by remember(input) { mutableStateOf<ConfirmDetailElement?>(null) }
    var isShowedBroadcastError by remember((state as? ConfirmState.BroadcastError)?.error) {
        mutableStateOf(state is ConfirmState.BroadcastError)
    }
    var isShowBottomSheetInfo by remember(state as? ConfirmState.Error) {
        mutableStateOf((state as? ConfirmState.Error)?.error is GemConfirmException.InsufficientNetworkFee)
    }

    LaunchedEffect(input, simulationResult) {
        if (input == null) {
            cancelAction()
            return@LaunchedEffect
        }
        viewModel.init(input, simulationResult)
    }

    BackHandler(handleSystemBack) {
        cancelAction()
    }

    val perpetualType by viewModel.perpetualType.collectAsStateWithLifecycle()
    Scene(
        title = confirmTitle(isExternalRequest, amountModel?.transactionType, perpetualType),
        closeIcon = isExternalRequest,
        onClose = { cancelAction() },
        mainAction = {
            MainActionButton(
                title = state.buttonLabel(),
                state = buttonState,
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
                    isPayment && simulation.headerAsset == null && state is ConfirmState.Prepare -> Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .alpha(0f)
                            .clearAndSetSemantics { },
                    ) {
                        AmountListHead(amount = "", icon = input.inputType.asset)
                    }
                    simulation.headerAsset != null -> {
                        val asset = requireNotNull(simulation.headerAsset)
                        val title = if (simulation.headerIsUnlimited) {
                            stringResource(R.string.simulation_header_unlimited_asset, asset.symbol)
                        } else {
                            simulation.headerValue?.toBigIntegerOrNull()
                                ?.let { ValueFormatter(style = ValueFormatter.Style.Full).string(it, asset) } ?: asset.symbol
                        }
                        AmountListHead(amount = title, icon = asset)
                    }
                    amountModel?.headerKind is GemTransactionHeaderKind.Swap -> {
                        val model = requireNotNull(amountModel)
                        SwapListHead(
                            fromAsset = model.fromAsset,
                            fromValue = model.fromAmount,
                            toAsset = requireNotNull(model.toAsset),
                            toValue = requireNotNull(model.toAmount),
                            currency = model.currency,
                        )
                    }

                    amountModel?.headerKind is GemTransactionHeaderKind.Nft -> amountModel?.nftAsset?.let { NftHead(it) }

                    amountModel?.headerKind is GemTransactionHeaderKind.Symbol || amountModel?.headerKind is GemTransactionHeaderKind.AssetImage -> {
                        val asset = amountModel?.asset
                        AmountListHead(
                            amount = asset?.symbol.orEmpty(),
                            icon = asset,
                        )
                    }

                    else -> AmountListHead(
                        amount = amountModel?.cryptoAmount ?: "",
                        equivalent = amountModel?.amountEquivalent?.takeIf { (amountModel?.headerKind as? GemTransactionHeaderKind.Amount)?.showsFiat != false },
                        icon = if (input?.inputType is GemTransactionInputType.Withdrawal) {
                            PerpetualConfig.depositAsset
                        } else {
                            amountModel?.asset
                        },
                    )
                }
            }
            val sectionSize = displayTransactionProperties.size + detailElements.size
            itemsIndexed(displayTransactionProperties) { index, item ->
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
                            val walletIcon = walletImageModel(context, item.walletImageUrl)
                                ?: walletItemIconModel(item.walletType, item.walletChain)
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
                val listPosition = ListPosition.getPosition(displayTransactionProperties.size + index, sectionSize)
                ConfirmDetailElementRow(
                    item = item,
                    listPosition = listPosition,
                    onClick = { selectedDetailElement = item },
                )
            }
            simulationWarningsContent(simulation.warnings)
            simulationPayloadFieldsContent(
                fields = simulation.primaryPayloadFields,
                addressNames = payloadAddressNames,
                onDetailsClick = simulation.secondaryPayloadFields
                    .takeIf { it.isNotEmpty() }
                    ?.let { { showSimulationDetails = true } },
            )
            confirmBalanceChangesContent(simulation.balanceChanges)
            item {
                feeModel?.let {
                    val feeInfo = InfoSheetEntity.NetworkFeeInfo(
                        feeAsset?.asset?.name.orEmpty(),
                        feeAsset?.asset?.symbol.orEmpty(),
                    )
                    when (it) {
                        FeeUIModel.Calculating -> PropertyItem(
                            title = { PropertyTitleText(R.string.transfer_network_fee, info = feeInfo) },
                            data = { Row(horizontalArrangement = Arrangement.End) { CircularProgressIndicator14() } },
                            listPosition = ListPosition.Single,
                        )

                        is FeeUIModel.FeeInfo -> PropertyNetworkFee(
                            networkTitle = it.feeAsset.name,
                            networkSymbol = it.feeAsset.symbol,
                            feeCrypto = it.cryptoAmount,
                            feeFiat = it.fiatAmount,
                            variantsAvailable = true,
                            showFeeAssetSymbol = feeAssets.any { asset -> asset.asset.id != it.feeAsset.id },
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
                ConfirmErrorInfo(
                    state = state,
                    fee = feeModel as? FeeUIModel.FeeInfo,
                    isShowBottomSheetInfo = isShowBottomSheetInfo,
                    onAcquireAsset = onAcquireAsset,
                )
            }
        }

        FeeDetails(
            isVisible = showSelectTxSpeed,
            currentFee = feeModel as? FeeUIModel.FeeInfo,
            selection = feeSelection,
            feeRates = feeRates,
            feeDetailsModel = viewModel::feeDetailsModel,
            feeAsset = feeAsset,
            feeAssets = feeAssets,
            onSelect = viewModel::changeFeeSelection,
            onSelectFeeAsset = viewModel::changeFeeAsset,
            onCancel = { showSelectTxSpeed = false },
        )

        ModalBottomSheet(
            isVisible = showSimulationDetails,
            onDismissRequest = { showSimulationDetails = false },
            skipPartiallyExpanded = true,
            title = stringResource(R.string.common_details),
        ) {
            LazyColumn {
                simulationPayloadDetailsContent(
                    primaryFields = simulation.primaryPayloadFields,
                    secondaryFields = simulation.secondaryPayloadFields,
                    addressNames = payloadAddressNames,
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
                Text((state as? ConfirmState.BroadcastError)?.error?.toBroadcastLabel() ?: stringResource(R.string.errors_error_occurred))
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
        is ConfirmState.FatalError -> stringResource(messageRes)
        ConfirmState.Prepare,
        ConfirmState.Ready,
        is ConfirmState.Result,
        ConfirmState.Sending -> stringResource(id = R.string.transfer_confirm)
    }
}

@Composable
fun Throwable.toPreloadLabel(): String = toConfirmLabel()
    ?: toGemNetworkError()?.localizedDescription()
    ?: "${stringResource(R.string.confirm_fee_error)}: ${stringResource(R.string.errors_unable_estimate_network_fee)}"

@Composable
fun Throwable.toBroadcastLabel(): String = toConfirmLabel()
    ?: toGemNetworkError()?.localizedDescription()
    ?: "${stringResource(R.string.errors_transfer_error)}: ${message ?: toString()}"

@Composable
private fun Throwable.toConfirmLabel(): String? = when (this) {
    is GemConfirmException.ScanMalicious -> stringResource(R.string.errors_scan_transaction_malicious_description)
    is GemConfirmException.ScanMemoRequired -> stringResource(R.string.errors_scan_transaction_memo_required, symbol)
    is GemConfirmException.InsufficientBalance -> stringResource(R.string.transfer_insufficient_balance, asset.toPrimitives().title.boldMarkdown())
    is GemConfirmException.InsufficientNetworkFee -> stringResource(R.string.transfer_insufficient_network_fee_balance, asset.toPrimitives().title.boldMarkdown())
    is GemConfirmException.MinimumAccountBalanceTooLow -> stringResource(
        R.string.transfer_minimum_account_balance,
        ValueFormatter(style = ValueFormatter.Style.Full).string(BigInteger(requirement.required), asset.toPrimitives()).boldMarkdown(),
    )
    is GemConfirmException.Offline -> GemNetworkError.Offline.localizedDescription()
    is GemConfirmException.Network -> msg
    is GemConfirmException.Broadcast -> "${stringResource(R.string.errors_transfer_error)}: $msg"
    is GemConfirmException.Sign -> when (error) {
        GemSignerError.DustThreshold -> stringResource(R.string.errors_dust_threshold_short)
        else -> stringResource(R.string.errors_transfer_error)
    }
    is GemConfirmException -> null
    else -> null
}

@Composable
private fun confirmTitle(
    isExternalRequest: Boolean,
    transactionType: TransactionType?,
    perpetualType: PerpetualType?,
): String = when {
    isExternalRequest -> stringResource(R.string.transfer_review_request)
    perpetualType != null -> perpetualType.title()
    else -> stringResource(transactionType?.titleRes() ?: R.string.transfer_title)
}

@StringRes
private fun TransactionType.titleRes(): Int = when (this) {
    TransactionType.EarnDeposit,
    TransactionType.StakeDelegate -> R.string.transfer_stake_title
    TransactionType.EarnWithdraw,
    TransactionType.StakeWithdraw -> R.string.transfer_withdraw_title
    TransactionType.StakeUndelegate -> R.string.transfer_unstake_title
    TransactionType.StakeRedelegate -> R.string.transfer_redelegate_title
    TransactionType.StakeRewards -> R.string.transfer_rewards_title
    TransactionType.Transfer,
    TransactionType.TransferNFT -> R.string.transfer_send_title
    TransactionType.Swap -> R.string.wallet_swap
    TransactionType.TokenApproval -> R.string.transfer_approve_title
    TransactionType.AssetActivation -> R.string.transfer_activate_asset_title
    TransactionType.SmartContractCall -> R.string.transfer_smart_contract_title
    TransactionType.PerpetualOpenPosition -> R.string.perpetual_position
    TransactionType.PerpetualClosePosition -> R.string.perpetual_close_position
    TransactionType.StakeFreeze -> R.string.transfer_freeze_title
    TransactionType.StakeUnfreeze -> R.string.transfer_unfreeze_title
    TransactionType.PerpetualModifyPosition -> R.string.perpetual_modify
}
