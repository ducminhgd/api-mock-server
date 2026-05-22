import type { AxiosRequestConfig } from 'axios'
import axiosInstance from '@/infrastructure/http/axiosInstance'

export const apiClient = {
  get<T>(url: string, config?: AxiosRequestConfig) {
    return axiosInstance.get<T>(url, config).then((r) => r.data)
  },
  post<T>(url: string, data?: unknown, config?: AxiosRequestConfig) {
    return axiosInstance.post<T>(url, data, config).then((r) => r.data)
  },
  put<T>(url: string, data?: unknown, config?: AxiosRequestConfig) {
    return axiosInstance.put<T>(url, data, config).then((r) => r.data)
  },
  patch<T>(url: string, data?: unknown, config?: AxiosRequestConfig) {
    return axiosInstance.patch<T>(url, data, config).then((r) => r.data)
  },
  delete<T>(url: string, config?: AxiosRequestConfig) {
    return axiosInstance.delete<T>(url, config).then((r) => r.data)
  },
}
