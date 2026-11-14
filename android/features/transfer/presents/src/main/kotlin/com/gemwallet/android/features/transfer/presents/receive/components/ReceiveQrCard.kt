package com.gemwallet.android.features.transfer.presents.receive.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.theme.WindowDimension
import com.gemwallet.android.ui.theme.isCompactDimension
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall

private val qrCardElevation = 3.dp
private val qrSize = 300.dp
private val qrSizeCompact = 220.dp
private val qrMinSize = 100.dp

@Composable
internal fun receiveQrWidth(): Dp = if (isCompactDimension(WindowDimension.Height)) qrSizeCompact else qrSize

@Composable
internal fun receiveQrPadding(): Dp = if (isCompactDimension(WindowDimension.Height)) paddingSmall else paddingDefault

@Composable
internal fun ReceiveQrCard(content: String, caption: String, onCopy: () -> Unit) {
    val imageSize = receiveQrWidth()
    val imagePadding = receiveQrPadding()
    ElevatedCard(
        modifier = Modifier.width(imageSize),
        elevation = CardDefaults.cardElevation(defaultElevation = qrCardElevation),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.background,
            contentColor = MaterialTheme.colorScheme.onSurface,
        ),
    ) {
        Box(
            modifier = Modifier
                .widthIn(qrMinSize, imageSize)
                .aspectRatio(1f)
                .padding(imagePadding)
                .clip(MaterialTheme.shapes.small)
                .background(Color.White)
                .clickable(onCopy)
                .padding(paddingHalfSmall),
        ) {
            rememberQRCodePainter(content = content, size = qrSize)?.let { painter ->
                Image(
                    modifier = Modifier.fillMaxSize(),
                    painter = painter,
                    contentDescription = stringResource(R.string.common_copy),
                    contentScale = ContentScale.FillWidth,
                )
            }
        }
        Text(
            modifier = Modifier
                .width(imageSize)
                .padding(horizontal = imagePadding)
                .clickable(onCopy)
                .testTag(caption),
            text = caption,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.secondary,
            fontWeight = FontWeight.Medium,
            style = MaterialTheme.typography.bodyMedium,
        )
        Spacer(modifier = Modifier.size(imagePadding))
    }
}
