package com.gemwallet.android.features.transfer.presents.confirm

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.transfer.presents.confirm.components.AddressRow
import com.gemwallet.android.features.transfer.presents.confirm.components.ConfirmErrorInfo
import com.gemwallet.android.features.transfer.presents.confirm.components.FeeDetails
import com.gemwallet.android.features.transfer.presents.confirm.components.confirmBalanceChangesContent
import com.gemwallet.android.features.transfer.viewmodels.confirm.ConfirmViewModel
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.AcquireAssetAction
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.ConfirmHeaderUIModel
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.ConfirmRowUIModel
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.RefreshOnTimer
import com.gemwallet.android.ui.components.WebView
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.AssetValueListHead
import com.gemwallet.android.ui.components.list_head.NftHead
import com.gemwallet.android.ui.components.list_head.SwapListHead
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.perpetual.PerpetualDetailsBottomSheet
import com.gemwallet.android.ui.components.perpetual.PerpetualDetailsSummaryItem
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.components.simulation.simulationPayloadDetailsContent
import com.gemwallet.android.ui.components.simulation.simulationPayloadFieldsContent
import com.gemwallet.android.ui.components.swap.SwapDetailsBottomSheet
import com.gemwallet.android.ui.components.swap.SwapDetailsSummaryItem
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemConfirmAction
import uniffi.gemstone.GemConfirmSection
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.SimulationResult

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConfirmScreen(
    input: ConfirmTransferInput? = null,
    simulationResult: SimulationResult? = null,
    finishAction: FinishConfirmAction,
    cancelAction: CancelAction,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    paymentAsset: AssetId? = null,
    onPaymentAssetConsumed: () -> Unit = {},
    onSelectPaymentAsset: (List<AssetId>) -> Unit = {},
    onOpenAddress: (ChainAddress) -> Unit,
    handleSystemBack: Boolean = false,
    viewModel: ConfirmViewModel = hiltViewModel(),
) {
    val refreshIntervalMillis by viewModel.refreshIntervalMillis.collectAsStateWithLifecycle()
    RefreshOnTimer(refreshIntervalMillis) {
        if (viewModel.screen.value.refreshes()) viewModel.fetch()
    }

    val context = LocalContext.current
    val transactionRows by viewModel.transactionRows.collectAsStateWithLifecycle()
    val feeModel by viewModel.feeUIModel.collectAsStateWithLifecycle()
    val feeListItem by viewModel.feeListItem.collectAsStateWithLifecycle()
    val acquireOptions by viewModel.acquireOptions.collectAsStateWithLifecycle()
    val feeItems by viewModel.feeItems.collectAsStateWithLifecycle()
    val balanceChangeRows by viewModel.balanceChangeRows.collectAsStateWithLifecycle()
    val loadError by viewModel.loadError.collectAsStateWithLifecycle()
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val payloadChain by viewModel.payloadChain.collectAsStateWithLifecycle()
    val acquireRequest by viewModel.acquireRequest.collectAsStateWithLifecycle()
    val feeInfo by viewModel.feeInfo.collectAsStateWithLifecycle()
    val executeErrorText by viewModel.executeErrorText.collectAsStateWithLifecycle()
    val isVerificationFailed by viewModel.isVerificationFailed.collectAsStateWithLifecycle()
    val buttonLabel by viewModel.buttonLabel.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()
    val header by viewModel.header.collectAsStateWithLifecycle()
    val feeSelectionUIModel by viewModel.feeSelectionUIModel.collectAsStateWithLifecycle()
    val feeAssets by viewModel.feeAssets.collectAsStateWithLifecycle()
    val showsFeeAssets by viewModel.showsFeeAssets.collectAsStateWithLifecycle()
    val feeAsset by viewModel.feeAsset.collectAsStateWithLifecycle()
    val detailElements by viewModel.detailElements.collectAsStateWithLifecycle()
    val title by viewModel.title.collectAsStateWithLifecycle()
    val isExternalRequest by viewModel.isExternalRequest.collectAsStateWithLifecycle()
    val verification by viewModel.verification.collectAsStateWithLifecycle()
    val isVerificationVisible by viewModel.isVerificationVisible.collectAsStateWithLifecycle()

    var showSelectTxSpeed by remember { mutableStateOf(false) }
    var showSimulationDetails by remember { mutableStateOf(false) }
    var isVerificationInfoVisible by remember { mutableStateOf(false) }
    var selectedDetailElement by remember(input) { mutableStateOf<ConfirmDetailElement?>(null) }
    val payload = sections.filterIsInstance<GemConfirmSection.Payload>().firstOrNull()
    val openPayloadAddress = payloadChain?.let { chain ->
        { address: String ->
            showSimulationDetails = false
            onOpenAddress(ChainAddress(chain, address))
        }
    }
    var isShowedBroadcastError by remember(executeErrorText) { mutableStateOf(executeErrorText != null) }
    val isShowBottomSheetInfo by viewModel.isErrorSheetVisible.collectAsStateWithLifecycle()

    LaunchedEffect(input, simulationResult) {
        if (input == null) {
            cancelAction()
            return@LaunchedEffect
        }
        viewModel.init(input.data, simulationResult, input.wallet)
    }

    LaunchedEffect(paymentAsset) {
        val assetId = paymentAsset ?: return@LaunchedEffect
        viewModel.changePaymentAsset(assetId)
        onPaymentAssetConsumed()
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
                title = buttonLabel,
                state = buttonState,
                onClick = {
                    when (viewModel.action()) {
                        GemConfirmAction.EXECUTE -> context.requestAuth(AuthRequest.Confirmation) {
                            viewModel.send(finishAction)
                        }

                        GemConfirmAction.LOAD, null -> viewModel.send(finishAction)
                    }
                },
            )
        },
    ) { paddingValues ->
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = paddingValues.calculateBottomPadding() + paddingDefault),
        ) {
            sections.forEach { section ->
                when (section) {
                    GemConfirmSection.Header -> {
                        item {
                            when (val model = header) {
                                is ConfirmHeaderUIModel.Placeholder -> AmountListHead(amount = "", icon = model.icon)

                                is ConfirmHeaderUIModel.ReservedSpace -> Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .alpha(0f)
                                        .clearAndSetSemantics { },
                                ) {
                                    AmountListHead(amount = "", icon = model.icon)
                                }

                                is ConfirmHeaderUIModel.Simulation -> AssetValueListHead(model.header)

                                is ConfirmHeaderUIModel.Swap -> SwapListHead(
                                    fromAsset = model.fromAsset,
                                    fromValueText = model.fromValueText,
                                    fromEquivalentText = model.fromEquivalentText,
                                    toAsset = model.toAsset,
                                    toValueText = model.toValueText,
                                    toEquivalentText = model.toEquivalentText,
                                )

                                is ConfirmHeaderUIModel.Nft -> NftHead(model.source)

                                is ConfirmHeaderUIModel.Symbol -> AmountListHead(amount = model.asset.symbol, icon = model.asset)

                                is ConfirmHeaderUIModel.Amount -> AmountListHead(amount = model.amount, equivalent = model.equivalent, icon = model.asset)

                                null -> Unit
                            }
                        }
                    }

                    is GemConfirmSection.Notice -> item { GemListRowView(row = section.row, listPosition = ListPosition.Single) }

                    is GemConfirmSection.Details -> {
                        val sectionSize = transactionRows.size + detailElements.size
                        itemsIndexed(transactionRows) { index, row ->
                            val listPosition = ListPosition.getPosition(index, sectionSize)
                            when (row) {
                                is ConfirmRowUIModel.Row -> GemListRowView(row = row.row, listPosition = listPosition)

                                is ConfirmRowUIModel.Item -> ListItem(model = row.model, listPosition = listPosition)

                                is ConfirmRowUIModel.Address -> AddressRow(
                                    row = row,
                                    listPosition = listPosition,
                                    onClick = { onOpenAddress(ChainAddress(row.chain, row.address)) },
                                )

                                is ConfirmRowUIModel.Validator -> AddressPropertyItem(
                                    title = row.title,
                                    displayText = row.name,
                                    copyValue = row.address,
                                    explorerLink = row.explorerLink,
                                    listPosition = listPosition,
                                    onClick = { onOpenAddress(ChainAddress(row.chain, row.address)) },
                                )

                                is ConfirmRowUIModel.PaymentAsset -> ListItem(
                                    model = row.model,
                                    listPosition = listPosition,
                                    modifier = if (row.selectable) Modifier.clickable { onSelectPaymentAsset(row.assetIds) } else Modifier,
                                    accessory = if (row.selectable) {
                                        { DataBadgeChevron() }
                                    } else {
                                        null
                                    },
                                )
                            }
                        }
                        itemsIndexed(detailElements) { index, item ->
                            val listPosition = ListPosition.getPosition(transactionRows.size + index, sectionSize)
                            ConfirmDetailElementRow(
                                item = item,
                                listPosition = listPosition,
                                onClick = { selectedDetailElement = item },
                            )
                        }
                    }

                    is GemConfirmSection.Warnings -> itemsPositioned(section.rows) { position, row -> GemListRowView(row = row, listPosition = position) }

                    is GemConfirmSection.Payload -> simulationPayloadFieldsContent(
                        fields = section.primary,
                        onAddressClick = openPayloadAddress,
                        onDetailsClick = section.secondary
                            .takeIf { it.isNotEmpty() }
                            ?.let { { showSimulationDetails = true } },
                    )

                    is GemConfirmSection.BalanceChanges -> confirmBalanceChangesContent(balanceChangeRows)

                    GemConfirmSection.NetworkFee, GemConfirmSection.Verification -> {
                        item {
                            feeListItem?.let {
                                val onSelect: (() -> Unit)? = when {
                                    verification != null -> viewModel::showVerification

                                    feeInfo != null && feeModel !is FeeUIModel.Unavailable -> {
                                        { showSelectTxSpeed = true }
                                    }

                                    else -> null
                                }
                                ListItem(
                                    model = it,
                                    listPosition = ListPosition.Single,
                                    modifier = if (onSelect != null) Modifier.clickable(onClick = onSelect) else Modifier,
                                    accessory = if (onSelect != null) {
                                        { DataBadgeChevron() }
                                    } else {
                                        null
                                    },
                                )
                            }
                        }
                    }

                    is GemConfirmSection.Error -> Unit
                }
            }
            item {
                ConfirmErrorInfo(
                    error = loadError,
                    acquireRequest = acquireRequest,
                    acquireOptions = acquireOptions,
                    isShowBottomSheetInfo = isShowBottomSheetInfo,
                    onDismissBottomSheetInfo = viewModel::dismissErrorSheet,
                    onDismissAcquire = viewModel::dismissAcquire,
                    onAcquireAsset = onAcquireAsset,
                )
            }
        }

        FeeDetails(
            isVisible = showSelectTxSpeed,
            currentFee = feeInfo,
            feeItems = feeItems,
            feeListItem = feeListItem,
            selection = feeSelectionUIModel,
            feeDetailsModel = viewModel::feeDetailsModel,
            feeAsset = feeAsset,
            feeAssets = feeAssets,
            showFeeAssets = showsFeeAssets,
            onSelectPriority = viewModel::changeFeePriority,
            onSelectCustom = viewModel::changeCustomFee,
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
                    primaryFields = payload?.primary.orEmpty(),
                    secondaryFields = payload?.secondary.orEmpty(),
                    onAddressClick = openPayloadAddress,
                )
            }
        }

        ConfirmDetailElementBottomSheet(
            item = selectedDetailElement,
            onDismiss = { selectedDetailElement = null },
        )

        ModalBottomSheet(
            isVisible = isVerificationVisible,
            onDismissRequest = viewModel::dismissVerification,
            expansion = SheetExpansion.Full,
            title = stringResource(R.string.info_payment_verification_title),
            actions = {
                IconButton(onClick = { isVerificationInfoVisible = true }) {
                    Icon(AppIcons.InfoOutlined, contentDescription = null)
                }
            },
        ) {
            verification?.let {
                WebView(
                    url = it.url,
                    bridge = viewModel.verificationBridge,
                    modifier = Modifier.fillMaxSize(),
                )
            }
        }

        InfoBottomSheet(
            item = GemInfoTopic.PaymentVerification.infoSheet().takeIf { isVerificationInfoVisible },
            onClose = { isVerificationInfoVisible = false },
        )
    }

    if (isVerificationFailed) {
        AlertDialog(
            onDismissRequest = viewModel::dismissVerificationError,
            confirmButton = {
                Button(viewModel::dismissVerificationError) { Text(stringResource(R.string.common_done)) }
            },
            text = {
                Text(stringResource(R.string.errors_error_occurred))
            },
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
                Text(executeErrorText ?: stringResource(R.string.errors_error_occurred))
            },
        )
    }
}

@Composable
private fun ConfirmDetailElementRow(item: ConfirmDetailElement, listPosition: ListPosition, onClick: () -> Unit) {
    when (item) {
        is ConfirmDetailElement.SwapDetails -> SwapDetailsSummaryItem(
            model = item.model,
            onClick = onClick,
            listPosition = listPosition,
        )

        is ConfirmDetailElement.PerpetualDetails -> PerpetualDetailsSummaryItem(
            details = item.details,
            onClick = onClick,
            listPosition = listPosition,
        )

        is ConfirmDetailElement.PerpetualModifyAutoclose -> GemListRowView(row = item.row, listPosition = listPosition)
    }
}

@Composable
private fun ConfirmDetailElementBottomSheet(item: ConfirmDetailElement?, onDismiss: () -> Unit) {
    SwapDetailsBottomSheet(
        isVisible = item is ConfirmDetailElement.SwapDetails,
        isLoading = false,
        model = (item as? ConfirmDetailElement.SwapDetails)?.model,
        onDismiss = onDismiss,
        showProviderSectionHeader = true,
    )
    PerpetualDetailsBottomSheet(
        isVisible = item is ConfirmDetailElement.PerpetualDetails,
        details = (item as? ConfirmDetailElement.PerpetualDetails)?.details,
        onDismiss = onDismiss,
    )
}
