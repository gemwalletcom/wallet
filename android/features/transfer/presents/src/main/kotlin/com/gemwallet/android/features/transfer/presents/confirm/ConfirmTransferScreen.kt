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
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.features.transfer.presents.confirm.components.ConfirmErrorInfo
import com.gemwallet.android.features.transfer.presents.confirm.components.FeeDetails
import com.gemwallet.android.features.transfer.presents.confirm.components.confirmBalanceChangesContent
import com.gemwallet.android.features.transfer.viewmodels.confirm.ConfirmTransferViewModel
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.ConfirmDetailsUIModel
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.GetAssetAction
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.RefreshOnTimer
import com.gemwallet.android.ui.components.WebView
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_head.TransactionListHead
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
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
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.style.icon
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemConfirmAction
import uniffi.gemstone.GemConfirmRowContent
import uniffi.gemstone.GemConfirmSection
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.SimulationResult

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConfirmTransferScreen(
    input: ConfirmTransferInput? = null,
    simulationResult: SimulationResult? = null,
    finishAction: FinishConfirmAction,
    cancelAction: CancelAction,
    onGetAsset: (GetAssetAction, AssetId) -> Unit,
    paymentAsset: AssetId? = null,
    onPaymentAssetConsumed: () -> Unit = {},
    onSelectPaymentAsset: (List<AssetId>) -> Unit = {},
    onOpenAddress: (ChainAddress) -> Unit,
    handleSystemBack: Boolean = false,
    viewModel: ConfirmTransferViewModel = hiltViewModel(),
) {
    val refreshIntervalMillis by viewModel.refreshIntervalMillis.collectAsStateWithLifecycle()
    RefreshOnTimer(refreshIntervalMillis) {
        if (viewModel.screen.value.refreshes()) viewModel.fetch()
    }

    val context = LocalContext.current
    val transactionRows by viewModel.transactionRows.collectAsStateWithLifecycle()
    val feeListItem by viewModel.feeListItem.collectAsStateWithLifecycle()
    val balanceChangeRows by viewModel.balanceChangeRows.collectAsStateWithLifecycle()
    val loadError by viewModel.loadError.collectAsStateWithLifecycle()
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val payloadChain by viewModel.payloadChain.collectAsStateWithLifecycle()
    val acquireRequest by viewModel.acquireRequest.collectAsStateWithLifecycle()
    val feeRow by viewModel.feeRow.collectAsStateWithLifecycle()
    val executeErrorText by viewModel.executeErrorText.collectAsStateWithLifecycle()
    val isVerificationFailed by viewModel.isVerificationFailed.collectAsStateWithLifecycle()
    val buttonLabel by viewModel.buttonLabel.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()
    val button by viewModel.button.collectAsStateWithLifecycle()
    val header by viewModel.header.collectAsStateWithLifecycle()
    val feeScreen by viewModel.feeScreen.collectAsStateWithLifecycle()
    val detailElements by viewModel.detailElements.collectAsStateWithLifecycle()
    val title by viewModel.title.collectAsStateWithLifecycle()
    val isExternalRequest by viewModel.isExternalRequest.collectAsStateWithLifecycle()
    val verification by viewModel.verification.collectAsStateWithLifecycle()
    val isVerificationVisible by viewModel.isVerificationVisible.collectAsStateWithLifecycle()

    var showSelectTxSpeed by remember { mutableStateOf(false) }
    var showSimulationDetails by remember { mutableStateOf(false) }
    var isVerificationInfoVisible by remember { mutableStateOf(false) }
    var selectedDetailElement by remember(input) { mutableStateOf<ConfirmDetailsUIModel?>(null) }
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
                icon = button.icon.icon(),
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
                            header?.let { header ->
                                Box(
                                    modifier = if (header.isReserved) {
                                        Modifier
                                            .fillMaxWidth()
                                            .alpha(0f)
                                            .clearAndSetSemantics { }
                                    } else {
                                        Modifier
                                    },
                                ) {
                                    TransactionListHead(header.header)
                                }
                            }
                        }
                    }

                    is GemConfirmSection.Notice -> item { GemListRowView(row = section.row, listPosition = ListPosition.Single) }

                    is GemConfirmSection.Details -> {
                        val sectionSize = transactionRows.size + detailElements.size
                        itemsIndexed(transactionRows) { index, row ->
                            val listPosition = ListPosition.getPosition(index, sectionSize)
                            when (row) {
                                is GemConfirmRowContent.Row -> GemListRowView(row = row.row, listPosition = listPosition)

                                is GemConfirmRowContent.Recipient -> AddressPropertyItem(
                                    row = row.row,
                                    listPosition = listPosition,
                                    onClick = { onOpenAddress(ChainAddress(row.row.chain.toChain(), row.row.address)) },
                                )

                                is GemConfirmRowContent.PaymentAsset -> ListItem(
                                    model = ListItemModel(title = row.title.text(context), subtitle = row.symbol),
                                    listPosition = listPosition,
                                    modifier = if (row.selectable) Modifier.clickable { onSelectPaymentAsset(row.assetIds.mapNotNull { it.toAssetId() }) } else Modifier,
                                    accessory = if (row.selectable) {
                                        { DataBadgeChevron() }
                                    } else {
                                        null
                                    },
                                )

                                GemConfirmRowContent.Details -> Unit
                            }
                        }
                        itemsIndexed(detailElements) { index, item ->
                            val listPosition = ListPosition.getPosition(transactionRows.size + index, sectionSize)
                            ConfirmDetailsRow(
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

                                    feeRow?.opensDetails == true -> {
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
                    isShowBottomSheetInfo = isShowBottomSheetInfo,
                    onDismissBottomSheetInfo = viewModel::dismissErrorSheet,
                    onDismissAcquire = viewModel::dismissAcquire,
                    onGetAsset = onGetAsset,
                )
            }
        }

        FeeDetails(
            isVisible = showSelectTxSpeed,
            screen = feeScreen,
            feeListItem = feeListItem,
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

        ConfirmDetailsSheet(
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
private fun ConfirmDetailsRow(item: ConfirmDetailsUIModel, listPosition: ListPosition, onClick: () -> Unit) {
    when (item) {
        is ConfirmDetailsUIModel.SwapDetails -> SwapDetailsSummaryItem(
            model = item.model,
            onClick = onClick,
            listPosition = listPosition,
        )

        is ConfirmDetailsUIModel.PerpetualDetails -> PerpetualDetailsSummaryItem(
            details = item.details,
            onClick = onClick,
            listPosition = listPosition,
        )

        is ConfirmDetailsUIModel.PerpetualModifyAutoclose -> GemListRowView(row = item.row, listPosition = listPosition)
    }
}

@Composable
private fun ConfirmDetailsSheet(item: ConfirmDetailsUIModel?, onDismiss: () -> Unit) {
    SwapDetailsBottomSheet(
        isVisible = item is ConfirmDetailsUIModel.SwapDetails,
        isLoading = false,
        model = (item as? ConfirmDetailsUIModel.SwapDetails)?.model,
        onDismiss = onDismiss,
        showProviderSectionHeader = true,
    )
    PerpetualDetailsBottomSheet(
        isVisible = item is ConfirmDetailsUIModel.PerpetualDetails,
        details = (item as? ConfirmDetailsUIModel.PerpetualDetails)?.details,
        onDismiss = onDismiss,
    )
}
