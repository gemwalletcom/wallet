package com.gemwallet.android.features.confirm.presents

import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.ui.components.image.ListItemImageView
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
import com.gemwallet.android.features.asset.presents.address.AddressDetailsSheet
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.confirm.presents.components.AddressRow
import com.gemwallet.android.features.confirm.presents.components.ConfirmErrorInfo
import com.gemwallet.android.features.confirm.presents.components.FeeDetails
import com.gemwallet.android.features.confirm.presents.components.confirmBalanceChangesContent
import com.gemwallet.android.features.confirm.presents.localization.string
import com.gemwallet.android.features.confirm.viewmodels.ConfirmViewModel
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireAssetAction
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmHeaderUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmRowUIModel
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.AssetValueListHead
import com.gemwallet.android.ui.components.list_head.NftHead
import com.gemwallet.android.ui.components.list_head.SwapListHead
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.perpetual.AutocloseSummaryRow
import com.gemwallet.android.ui.components.perpetual.PerpetualDetailsBottomSheet
import com.gemwallet.android.ui.components.perpetual.PerpetualDetailsSummaryItem
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.components.simulation.simulationPayloadDetailsContent
import com.gemwallet.android.ui.components.simulation.simulationPayloadFieldsContent
import com.gemwallet.android.ui.components.simulation.simulationWarningsContent
import com.gemwallet.android.ui.components.swap.SwapDetailsBottomSheet
import com.gemwallet.android.ui.components.swap.SwapDetailsSummaryItem
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.SimulationResult

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConfirmScreen(
    input: ConfirmTransferInput? = null,
    simulationResult: SimulationResult? = null,
    finishAction: FinishConfirmAction,
    cancelAction: CancelAction,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    handleSystemBack: Boolean = false,
    viewModel: ConfirmViewModel = hiltViewModel(),
) {
    val context = LocalContext.current
    val transactionRows by viewModel.transactionRows.collectAsStateWithLifecycle()
    val feeModel by viewModel.feeUIModel.collectAsStateWithLifecycle()
    val feeListItem by viewModel.feeListItem.collectAsStateWithLifecycle()
    val acquireOptions by viewModel.acquireOptions.collectAsStateWithLifecycle()
    val feeItems by viewModel.feeItems.collectAsStateWithLifecycle()
    val balanceChangeRows by viewModel.balanceChangeRows.collectAsStateWithLifecycle()
    val loadError by viewModel.loadError.collectAsStateWithLifecycle()
    val acquireRequest by viewModel.acquireRequest.collectAsStateWithLifecycle()
    val feeValue by viewModel.feeValue.collectAsStateWithLifecycle()
    val executeErrorText by viewModel.executeErrorText.collectAsStateWithLifecycle()
    val buttonLabel by viewModel.buttonLabel.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()
    val header by viewModel.header.collectAsStateWithLifecycle()
    val feeSelectionUIModel by viewModel.feeSelectionUIModel.collectAsStateWithLifecycle()
    val feeAssets by viewModel.feeAssets.collectAsStateWithLifecycle()
    val feeAsset by viewModel.feeAsset.collectAsStateWithLifecycle()
    val simulation by viewModel.simulation.collectAsStateWithLifecycle()
    val detailElements by viewModel.detailElements.collectAsStateWithLifecycle()
    val title by viewModel.title.collectAsStateWithLifecycle()
    val isExternalRequest by viewModel.isExternalRequest.collectAsStateWithLifecycle()

    var showSelectTxSpeed by remember { mutableStateOf(false) }
    var showSimulationDetails by remember { mutableStateOf(false) }
    var selectedDetailElement by remember(input) { mutableStateOf<ConfirmDetailElement?>(null) }
    var selectedAddress by remember(input) { mutableStateOf<ChainAddress?>(null) }
    var isShowedBroadcastError by remember(executeErrorText) { mutableStateOf(executeErrorText != null) }
    val isShowBottomSheetInfo by viewModel.isNetworkFeeSheetVisible.collectAsStateWithLifecycle()

    LaunchedEffect(input, simulationResult) {
        if (input == null) {
            cancelAction()
            return@LaunchedEffect
        }
        viewModel.init(input.data, simulationResult)
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
                when (val model = header) {
                    is ConfirmHeaderUIModel.Placeholder -> Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .alpha(0f)
                            .clearAndSetSemantics { },
                    ) {
                        AmountListHead(amount = "", icon = model.asset)
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
                    is ConfirmHeaderUIModel.Nft -> NftHead(model.nftAsset)
                    is ConfirmHeaderUIModel.Symbol -> AmountListHead(amount = model.asset.symbol, icon = model.asset)
                    is ConfirmHeaderUIModel.Amount -> AmountListHead(amount = model.amount, equivalent = model.equivalent, icon = model.asset)
                    null -> Unit
                }
            }
            val sectionSize = transactionRows.size + detailElements.size
            itemsIndexed(transactionRows) { index, row ->
                val listPosition = ListPosition.getPosition(index, sectionSize)
                when (row) {
                    is ConfirmRowUIModel.Item -> ListItem(
                        model = row.model,
                        listPosition = listPosition,
                        accessory = row.trailingImage?.let { image ->
                            { DataBadgeChevron(isShowChevron = false) { ListItemImageView(image = image, size = smallIconSize) } }
                        },
                    )
                    is ConfirmRowUIModel.Address -> AddressRow(
                        row = row,
                        listPosition = listPosition,
                        onClick = { selectedAddress = ChainAddress(row.chain, row.address) },
                    )
                    is ConfirmRowUIModel.Validator -> AddressPropertyItem(
                        title = row.title,
                        displayText = row.name,
                        copyValue = row.address,
                        explorerLink = row.explorerLink,
                        listPosition = listPosition,
                        onClick = { selectedAddress = ChainAddress(row.chain, row.address) },
                    )
                    is ConfirmRowUIModel.Network -> PropertyNetworkItem(chain = row.chain, value = row.name, listPosition = listPosition)
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
            simulationWarningsContent(simulation.warnings)
            simulationPayloadFieldsContent(
                fields = simulation.primaryPayloadFields,
                onDetailsClick = simulation.secondaryPayloadFields
                    .takeIf { it.isNotEmpty() }
                    ?.let { { showSimulationDetails = true } },
            )
            confirmBalanceChangesContent(balanceChangeRows)
            item {
                feeListItem?.let {
                    val canSelectFee = feeModel is FeeUIModel.FeeInfo
                    ListItem(
                        model = it,
                        listPosition = ListPosition.Single,
                        modifier = if (canSelectFee) Modifier.clickable { showSelectTxSpeed = true } else Modifier,
                        accessory = if (canSelectFee) {
                            { DataBadgeChevron() }
                        } else {
                            null
                        },
                    )
                }
            }
            item {
                ConfirmErrorInfo(
                    error = loadError,
                    acquireRequest = acquireRequest,
                    acquireOptions = acquireOptions,
                    isShowBottomSheetInfo = isShowBottomSheetInfo,
                    onDismissBottomSheetInfo = viewModel::dismissNetworkFeeSheet,
                    onDismissAcquire = viewModel::dismissAcquire,
                    onAcquireAsset = onAcquireAsset,
                )
            }
        }

        FeeDetails(
            isVisible = showSelectTxSpeed,
            currentFee = feeModel as? FeeUIModel.FeeInfo,
            feeItems = feeItems,
            feeListItem = feeListItem,
            selection = feeSelectionUIModel,
            feeDetailsModel = viewModel::feeDetailsModel,
            feeAsset = feeAsset,
            feeAssets = feeAssets,
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
                    primaryFields = simulation.primaryPayloadFields,
                    secondaryFields = simulation.secondaryPayloadFields,
                    )
            }
        }

        ConfirmDetailElementBottomSheet(
            item = selectedDetailElement,
            onDismiss = { selectedDetailElement = null },
        )

        AddressDetailsSheet(
            chainAddress = selectedAddress,
            onDismiss = { selectedAddress = null },
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
