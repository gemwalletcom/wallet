package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.Text
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.listItemSegment
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.space0

private const val TEXT_MESSAGE_BLOCK_LENGTH = 1024

fun LazyListScope.textMessageContent(message: String) {
    item { SubheaderItem(R.string.sign_message_message) }
    itemsPositioned(textMessageBlocks(message)) { position, block ->
        Text(
            modifier = Modifier
                .fillMaxWidth()
                .listItemSegment(position)
                .padding(
                    start = paddingDefault,
                    end = paddingDefault,
                    top = if (position == ListPosition.First || position == ListPosition.Single) paddingDefault else space0,
                    bottom = if (position == ListPosition.Last || position == ListPosition.Single) paddingDefault else space0,
                ),
            text = block,
        )
    }
}

fun textMessageBlocks(message: String): List<String> = message.lines().flatMap(::lineBlocks)

private fun lineBlocks(line: String): List<String> {
    val starts = generateSequence(0) { start -> blockEnd(line, start).takeIf { it < line.length } }.toList()
    return (starts + line.length).zipWithNext(line::substring)
}

private fun blockEnd(line: String, start: Int): Int {
    val limit = start + TEXT_MESSAGE_BLOCK_LENGTH
    if (limit >= line.length) return line.length
    val space = line.lastIndexOf(' ', limit - 1)
    return when {
        space >= start -> space + 1
        line[limit - 1].isHighSurrogate() -> limit - 1
        else -> limit
    }
}
