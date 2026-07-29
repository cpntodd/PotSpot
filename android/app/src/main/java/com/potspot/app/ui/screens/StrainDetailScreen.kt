package com.potspot.app.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.potspot.app.data.remote.ApiClient
import com.potspot.app.data.remote.StrainDetailDto
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun StrainDetailScreen(strainId: String, onBack: () -> Unit) {
    var strain by remember { mutableStateOf<StrainDetailDto?>(null) }
    val scope = rememberCoroutineScope()

    LaunchedEffect(strainId) {
        scope.launch {
            strain = try {
                ApiClient.api.getStrainDetail(strainId)
            } catch (_: Exception) { null }
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(strain?.name ?: "Loading...") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back")
                    }
                }
            )
        }
    ) { padding ->
        strain?.let { s ->
            Column(
                modifier = Modifier
                    .padding(padding)
                    .verticalScroll(rememberScrollState())
                    .padding(16.dp)
            ) {
                Text(s.name, style = MaterialTheme.typography.headlineMedium)
                Spacer(Modifier.height(8.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(s.type.uppercase())
                    s.thcPercentage?.let { Text("THC: $it%") }
                    s.cbdPercentage?.let { Text("CBD: $it%") }
                }
                s.averageRating?.let {
                    Text("★ ${"%.1f".format(it)} (${s.ratingCount} ratings)")
                }
                Spacer(Modifier.height(16.dp))
                s.description?.let {
                    Text(it, style = MaterialTheme.typography.bodyLarge)
                    Spacer(Modifier.height(16.dp))
                }
                if (s.terpenes.isNotEmpty()) {
                    Text("Terpenes", style = MaterialTheme.typography.titleMedium)
                    Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                        s.terpenes.forEach { t ->
                            AssistChip(onClick = {}, label = { Text("${t.icon} ${t.name}") })
                        }
                    }
                    Spacer(Modifier.height(16.dp))
                }
                if (s.effects.isNotEmpty()) {
                    Text("Effects", style = MaterialTheme.typography.titleMedium)
                    s.effects.groupBy { it.category }.forEach { (category, effects) ->
                        Text(category, style = MaterialTheme.typography.labelMedium)
                        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            effects.forEach { e ->
                                SuggestionChip(onClick = {}, label = { Text(e.name) })
                            }
                        }
                    }
                }
            }
        } ?: run {
            Box(modifier = Modifier.padding(padding).fillMaxSize()) {
                CircularProgressIndicator()
            }
        }
    }
}
