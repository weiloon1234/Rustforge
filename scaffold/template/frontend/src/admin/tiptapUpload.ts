import { api } from "@admin/api";
import type { ApiResponse } from "@shared/types";

interface AdminTiptapImageUploadOutput {
  folder: string;
  path: string;
  url: string;
  content_type: string;
  size: number;
  width: number | null;
  height: number | null;
}

export async function uploadAdminTiptapImage(file: File, folder: string): Promise<{ url: string; path: string }> {
  const formData = new FormData();
  formData.append("folder", folder);
  formData.append("fileUpload", file);

  const response = await api.post<ApiResponse<AdminTiptapImageUploadOutput>>(
    "/uploads/tiptap-image",
    formData,
  );
  const payload = response.data?.data;
  if (!payload || !payload.url || !payload.path) {
    throw new Error();
  }
  return {
    url: payload.url,
    path: payload.path,
  };
}
