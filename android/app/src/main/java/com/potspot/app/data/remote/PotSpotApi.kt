package com.potspot.app.data.remote

import retrofit2.http.*

interface PotSpotApi {
    // Auth
    @POST("api/v1/auth/login")
    suspend fun login(@Body request: LoginRequest): TokenResponse

    // Catalog
    @GET("api/v1/strains")
    suspend fun getStrains(
        @Query("page") page: Long = 1,
        @Query("per_page") perPage: Long = 50,
        @Query("sort") sort: String = "newest",
        @Query("q") query: String? = null,
        @Query("type") type: String? = null,
    ): StrainListResponse

    @GET("api/v1/strains/{id}")
    suspend fun getStrainDetail(@Path("id") id: String): StrainDetailDto

    @GET("api/v1/strains/{id}/similar")
    suspend fun getSimilarStrains(@Path("id") id: String): List<StrainSummaryDto>

    // Vault
    @GET("api/v1/vault")
    suspend fun getVault(): VaultResponse

    @POST("api/v1/vault/save/{strainId}")
    suspend fun saveStrain(@Path("strainId") strainId: String)

    @DELETE("api/v1/vault/save/{strainId}")
    suspend fun unsaveStrain(@Path("strainId") strainId: String)

    // Terpenes & Effects (catalog)
    @GET("api/v1/strains")
    suspend fun getFullCatalog(
        @Query("page") page: Long = 1,
        @Query("per_page") perPage: Long = 200,
    ): StrainListResponse
}
