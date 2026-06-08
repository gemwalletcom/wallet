// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
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

    var placeholder: String { "Message" }

    var canSend: Bool { text.trimmingCharacters(in: .whitespacesAndNewlines).isNotEmpty }

    func send() {
        let content = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard content.isNotEmpty else { return }
        onSendText(content)
        text = ""
    }

    func sendSelectedImages() {
        guard selectedItems.isNotEmpty else { return }
        let items = selectedItems
        selectedItems = []
        onSendImages(items)
    }
}
