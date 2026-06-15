// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import PhotosUI
import Primitives
import SwiftUI

@Observable
@MainActor
final class SupportMessageInputBarViewModel {
    var text: String = ""
    var selectedItems: [PhotosPickerItem] = []

    private let onSendText: (String) -> Void
    private let onSendImages: ([PhotosPickerItem]) -> Void

    init(
        onSendText: @escaping (String) -> Void,
        onSendImages: @escaping ([PhotosPickerItem]) -> Void,
    ) {
        self.onSendText = onSendText
        self.onSendImages = onSendImages
    }

    var placeholder: String { Localized.Support.messagePlaceholder }
    var canSend: Bool { trimmedText.isNotEmpty }
    private var trimmedText: String { text.trimmingCharacters(in: .whitespacesAndNewlines) }

    func send() {
        guard canSend else { return }
        onSendText(trimmedText)
        text = ""
    }

    func sendSelectedImages() {
        guard selectedItems.isNotEmpty else { return }
        let items = selectedItems
        selectedItems = []
        onSendImages(items)
    }
}
