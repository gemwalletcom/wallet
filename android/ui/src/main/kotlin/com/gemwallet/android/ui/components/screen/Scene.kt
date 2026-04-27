package com.gemwallet.android.ui.components.screen

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.theme.SceneSizing
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WindowDimension
import com.gemwallet.android.ui.theme.isCompactDimension
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.sceneContentPadding

@Composable
fun Scene(
    title: String,
    backHandle: Boolean = false,
    padding: PaddingValues = PaddingValues(horizontal = 0.dp),
    onClose: (() -> Unit)? = null,
    closeIcon: Boolean = false,
    actions: @Composable RowScope.() -> Unit = {},
    mainAction: (@Composable () -> Unit)? = null,
    mainActionPadding: PaddingValues = if (isCompactDimension(WindowDimension.Height)) {
        PaddingValues(horizontal = sceneContentPadding())
    } else {
        PaddingValues(
            start = sceneContentPadding(),
            top = paddingDefault,
            end = sceneContentPadding(),
            bottom = paddingDefault
        )
    },
    snackbar: SnackbarHostState? = null,
    navigationBarPadding: Boolean = true,
    content: @Composable ColumnScope.(PaddingValues) -> Unit,
) {
    Scene(
        titleContent = {
            Text(
                modifier = Modifier,
                text = title,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        },
        backHandle = backHandle,
        contentPadding = padding,
        onClose = onClose,
        closeIcon = closeIcon,
        actions = actions,
        mainAction = mainAction,
        mainActionPadding = mainActionPadding,
        snackbar = snackbar,
        navigationBarPadding = navigationBarPadding,
        content = content,
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun Scene(
    titleContent: @Composable () -> Unit,
    backHandle: Boolean = false,
    contentPadding: PaddingValues = PaddingValues(horizontal = 0.dp),
    onClose: (() -> Unit)? = null,
    closeIcon: Boolean = false,
    actions: @Composable RowScope.() -> Unit = {},
    mainAction: (@Composable () -> Unit)? = null,
    mainActionPadding: PaddingValues = PaddingValues(horizontal = sceneContentPadding(), vertical = paddingDefault),
    snackbar: SnackbarHostState? = null,
    navigationBarPadding: Boolean = true,
    progress: (() -> Float)? = null,
    content: @Composable ColumnScope.(PaddingValues) -> Unit,
) {
    BackHandler(backHandle) {
        onClose?.invoke()
    }
    Scaffold(
        modifier = Modifier.imePadding(),
        containerColor = MaterialTheme.colorScheme.surface,
        topBar = {
            Box(
                modifier = Modifier.fillMaxWidth()
            ) {
                CenterAlignedTopAppBar(
                    title = titleContent,
                    navigationIcon = {
                        if (onClose != null) {
                            IconButton(onClick = onClose) {
                                Icon(
                                    imageVector = if (closeIcon) Icons.Default.Close else Icons.AutoMirrored.Filled.ArrowBack,
                                    contentDescription = null,
                                )
                            }
                        }
                    },
                    actions = actions,
                )
                progress?.let {
                    LinearProgressIndicator(
                        modifier = Modifier.fillMaxWidth(),
                        progress = { it() },
                        trackColor = Color.Transparent,
                    )
                }
            }
        },
        bottomBar = {
            if (mainAction != null) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(MaterialTheme.colorScheme.surface)
                        .navigationBarsPadding(),
                    contentAlignment = Alignment.Center,
                ) {
                    Box(
                        modifier = Modifier
                            .widthIn(max = SceneSizing.buttonMaxWidth)
                            .padding(mainActionPadding)
                    ) {
                        mainAction()
                    }
                }
            }
        },
        snackbarHost = {
            if (snackbar != null) {
                SnackbarHost(hostState = snackbar) { data ->
                    Snackbar(
                        snackbarData = data,
                        containerColor = MaterialTheme.colorScheme.scrim,
                        contentColor = MaterialTheme.colorScheme.onSurface,
                        actionContentColor = MaterialTheme.colorScheme.primary,
                    )
                }
            }
        }
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .padding(
                    top = paddingValues.calculateTopPadding(),
                    bottom = if (navigationBarPadding) paddingValues.calculateBottomPadding() else 0.dp,
                )
                .fillMaxSize(),
        ) {
            Column(
                modifier = Modifier.fillMaxSize(),
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(contentPadding)
                        .weight(1f),
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth(),
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        content(paddingValues)
                        Spacer16()
                    }
                }
            }
        }
    }
}
