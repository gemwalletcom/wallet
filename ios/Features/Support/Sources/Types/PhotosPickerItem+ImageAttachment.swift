// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PhotosUI
import SupportChatService
import SwiftUI
import UIKit

private let imageCompressionQuality = 0.9
private let imageFileExtension = "jpg"
private let imageMimeType = "image/jpeg"

extension PhotosPickerItem {
    func imageAttachment() async throws -> ImageAttachment? {
        guard let data = try await loadTransferable(type: Data.self) else { return nil }
        guard let image = UIImage(data: data), let jpegData = image.compress(compressionQuality: imageCompressionQuality) else { return nil }

        return ImageAttachment(
            data: jpegData,
            fileName: "image-\(UUID().uuidString).\(imageFileExtension)",
            mimeType: imageMimeType,
        )
    }
}
