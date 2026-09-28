// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import PhotosUI
import SwiftUI
import UIKit

extension PhotosPickerItem {
    func imageAttachment() async throws -> Data? {
        guard let data = try await loadTransferable(type: Data.self) else { return nil }
        guard let image = UIImage(data: data) else { return nil }
        return image
            .fitting(maxDimension: CGFloat(GemConstants.supportAttachmentMaxDimension))
            .compress(compressionQuality: CGFloat(GemConstants.supportAttachmentJpegQuality) / 100)
    }
}

extension UIImage {
    func fitting(maxDimension: CGFloat) -> UIImage {
        let longest = max(size.width, size.height)
        guard longest > maxDimension else { return self }
        let scale = maxDimension / longest
        let target = CGSize(width: (size.width * scale).rounded(.down), height: (size.height * scale).rounded(.down))
        let format = UIGraphicsImageRendererFormat.default()
        format.scale = 1
        return UIGraphicsImageRenderer(size: target, format: format).image { _ in
            draw(in: CGRect(origin: .zero, size: target))
        }
    }
}
