package com.gemwallet.android.features.confirm.presents

import com.gemwallet.android.ui.components.screen.SheetExpansion
import androidx.activity.compose.BackHandler
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
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.features.confirm.presents.localization.buttonLabel
import com.gemwallet.android.features.confirm.presents.localization.string
import com.gemwallet.android.features.confirm.presents.localization.toBroadcastLabel
import uniffi.gemstone.GemConfirmButtonKind
import uniffi.gemstone.GemConfirmButtonState
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmStage
import uniffi.gemstone.GemConfirmScreen
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.features.confirm.presents.components.ConfirmErrorInfo
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.features.confirm.presents.components.FeeDetails
import com.gemwallet.android.features.confirm.presents.components.PropertyDestination
import com.gemwallet.android.features.confirm.viewmodels.ConfirmViewModel
import com.gemwallet.android.model.AuthRequest
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemTransactionHeaderKind
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.perpetual.AutocloseSummaryRow
import com.gemwallet.android.ui.components.perpetual.PerpetualDetailsBottomSheet
import com.gemwallet.android.ui.components.perpetual.PerpetualDetailsSummaryItem
import com.wallet.core.primitives.AssetId
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.image.walletImageModel
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.AssetValueListHead
import com.gemwallet.android.ui.components.list_head.NftHead
import com.gemwallet.android.ui.components.list_head.SwapListHead
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkFee
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.transaction.getTitle
import com.gemwallet.android.ui.components.list_item.iconModel
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator14
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.simulation.simulationPayloadDetailsContent
import com.gemwallet.android.ui.components.simulation.simulationPayloadFieldsContent
import com.gemwallet.android.ui.components.simulation.simulationWarningsContent
import com.gemwallet.android.ui.components.swap.SwapDetailsBottomSheet
import com.gemwallet.android.ui.components.swap.SwapDetailsSummaryItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.features.confirm.presents.components.confirmBalanceChangesContent
import uniffi.gemstone.SimulationResult

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
    val screen by viewModel.screen.collectAsStateWithLifecycle()
    val networkFeeBuyAmount by viewModel.networkFeeBuyAmount.collectAsStateWithLifecycle()
    val feeAssets by viewModel.feeAssets.collectAsStateWithLifecycle()
    val feeAsset by viewModel.feeAsset.collectAsStateWithLifecycle()
    val feeSelection by viewModel.feeSelection.collectAsStateWithLifecycle()
    val simulation by viewModel.simulation.collectAsStateWithLifecycle()
    val detailElements by viewModel.detailElements.collectAsStateWithLifecycle()
    val payloadAddressNames by viewModel.payloadAddressNames.collectAsStateWithLifecycle()
    val button by viewModel.button.collectAsStateWithLifecycle()
    val assetPrice by viewModel.assetPrice.collectAsStateWithLifecycle()
    val title by viewModel.title.collectAsStateWithLifecycle()
    val isExternalRequest by viewModel.isExternalRequest.collectAsStateWithLifecycle()
    val isPayment by viewModel.isPaymentRequest.collectAsStateWithLifecycle()
    val headerAsset by viewModel.headerAsset.collectAsStateWithLifecycle()
    val displayTransactionProperties = transactionProperties

    var showSelectTxSpeed by remember { mutableStateOf(false) }
    var showSimulationDetails by remember { mutableStateOf(false) }
    var selectedDetailElement by remember(input) { mutableStateOf<ConfirmDetailElement?>(null) }
    val executeError = screen.failure?.takeIf { it.stage == GemConfirmStage.EXECUTE }?.error
    var isShowedBroadcastError by remember(executeError) { mutableStateOf(executeError != null) }
    val isShowBottomSheetInfo by viewModel.isNetworkFeeSheetVisible.collectAsStateWithLifecycle()

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

    Scene(
        title = title?.string().orEmpty(),
        closeIcon = isExternalRequest,
        onClose = { cancelAction() },
        mainAction = {
            MainActionButton(
                title = screen.buttonLabel(button.kind),
                state = button.state.toButtonState(),
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
                    isPayment && simulation.header == null && screen.phase == GemConfirmPhase.LOADING -> Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .alpha(0f)
                            .clearAndSetSemantics { },
                    ) {
                        AmountListHead(amount = "", icon = headerAsset)
                    }
                    simulation.header != null -> AssetValueListHead(requireNotNull(simulation.header))
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
                        icon = headerAsset,
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
                    is ConfirmProperty.Network -> PropertyNetworkItem(chain = item.chain, value = item.name, listPosition = listPosition)
                    is ConfirmProperty.Source -> PropertyItem(
                        title = { PropertyTitleText(R.string.common_wallet) },
                        data = {
                            val walletIcon = walletImageModel(context, item.walletRow.imageUrl)
                                ?: item.walletRow.placeholder.iconModel()
                            PropertyDataText(
                                text = item.walletRow.name,
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
                    failure = screen.failure,
                    fee = feeModel as? FeeUIModel.FeeInfo,
                    isShowBottomSheetInfo = isShowBottomSheetInfo,
                    onDismissBottomSheetInfo = viewModel::dismissNetworkFeeSheet,
                    assetPrice = assetPrice,
                    acquireFlow = viewModel::acquireFlow,
                    networkFeeBuyAmount = networkFeeBuyAmount,
                    onAcquireAsset = onAcquireAsset,
                )
            }
        }

        FeeDetails(
            isVisible = showSelectTxSpeed,
            currentFee = feeModel as? FeeUIModel.FeeInfo,
            selection = feeSelection,
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
            expansion = SheetExpansion.Full,
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
                Text(executeError?.toBroadcastLabel() ?: stringResource(R.string.errors_error_occurred))
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
    SwapDetailsBottomSheet(
        isVisible = item is ConfirmDetailElement.SwapDetails,
        isLoading = false,
        model = (item as? ConfirmDetailElement.SwapDetails)?.model,
        onDismiss = onDismiss,
        showProviderSectionHeader = true,
    )
    PerpetualDetailsBottomSheet(
        isVisible = item is ConfirmDetailElement.PerpetualDetails,
        model = (item as? ConfirmDetailElement.PerpetualDetails)?.model,
        onDismiss = onDismiss,
    )
}

private fun GemConfirmButtonState.toButtonState(): ButtonState = when (this) {
    GemConfirmButtonState.DISABLED -> ButtonState.Disabled
    GemConfirmButtonState.LOADING -> ButtonState.Loading
    GemConfirmButtonState.ENABLED -> ButtonState.Enabled
}
