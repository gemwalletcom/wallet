package com.gemwallet.android.features.transactions.presents.transaction

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.transactions.viewmodels.TransactionViewModel
import com.gemwallet.android.features.transactions.viewmodels.models.TransactionItemUIModel
import com.gemwallet.android.features.transactions.viewmodels.models.chain
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.shareText

@Composable
fun TransactionScreen(onAction: (TransactionAction.Navigation) -> Unit, viewModel: TransactionViewModel = hiltViewModel()) {
    val transaction by viewModel.data.collectAsStateWithLifecycle()
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val headerTarget by viewModel.headerTarget.collectAsStateWithLifecycle()
    var isShowFeeDetails by remember { mutableStateOf(false) }
    val context = LocalContext.current

    fun onShare(url: String, name: String) {
        context.shareText(subject = null, text = url, chooserTitle = name)
    }

    val model = transaction
    if (model == null) {
        LoadingScene(
            title = "",
            onCancel = { onAction(TransactionAction.Close) },
        )
        return
    }

    TransactionScene(
        title = model.title.string(context),
        sections = sections,
        headerTarget = headerTarget,
        chain = model.chain(),
        onAction = {
            when (it) {
                TransactionAction.Share -> onShare(model.explorer.link, model.explorer.name)
                TransactionAction.ShowFeeDetails -> isShowFeeDetails = true
                is TransactionAction.Navigation -> onAction(it)
            }
        },
    )

    FeeDetailsDialog(
        isVisible = isShowFeeDetails,
        model = sections.flatMap { it.items }.firstNotNullOfOrNull { (it as? TransactionItemUIModel.Fee)?.model },
    ) { isShowFeeDetails = false }
}
