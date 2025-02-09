<script setup lang="ts">
import { ref, reactive, nextTick } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { FolderOpened, Connection, Check, Link } from "@element-plus/icons-vue";
import { shortFileName, useDynamicFormHeight } from "@/utils/utils";
import { catContent, useMarkdown } from "@/utils/markdown";

const [
  columns,
  selectedFiles,
  originalColumns,
  isLoading,
  completed,
  result,
  infoDialog
] = [ref(""), ref([]), ref([]), ref(false), ref(false), ref(null), ref(false)];
const data = reactive({
  filePath: "",
  fileFormats: ["*"],
  mode: "Memory",
  skipRows: "0",
  useCols: ""
});
const { formHeight } = useDynamicFormHeight(181);

async function selectFile() {
  columns.value = "";
  selectedFiles.value = [];
  originalColumns.value = [];
  completed.value = false;
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "",
        extensions: data.fileFormats
      }
    ]
  });
  if (Array.isArray(selected)) {
    data.filePath = selected.join("|").toString();
    const rows = selected.filter((row: any) => row.trim() !== "");
    selectedFiles.value = rows.map((file: any) => {
      return { filename: shortFileName(file) };
    });
  } else if (selected === null) {
    return;
  } else {
    data.filePath = selected;
  }

  const headers: string[] = await invoke("get_cat_headers", {
    path: data.filePath,
    skipRows: data.skipRows
  });

  if (JSON.stringify(headers).startsWith("get header error:")) {
    throw JSON.stringify(headers).toString();
  }

  originalColumns.value = headers.map(header => ({
    label: header,
    value: header
  }));
}

// invoke concat
async function concatData() {
  if (data.filePath === "") {
    ElNotification({
      title: "File not found",
      message: "未选择文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  const outputPath = await save({
    title: "Export",
    defaultPath: `cat_${new Date().getTime()}`,
    filters: [
      { name: "CSV", extensions: ["csv"] },
      { name: "Excel", extensions: ["xlsx"] }
    ]
  });

  if (outputPath === "" || outputPath === null) {
    ElNotification({
      title: "File not found",
      message: "未选择保存文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  isLoading.value = true;

  const saveFileType = outputPath.split(".").pop();
  const useCols = Object.values(columns.value).join("|");

  try {
    const res: string = await invoke("concat", {
      filePath: data.filePath,
      outputPath: outputPath,
      fileType: saveFileType,
      mode: data.mode,
      skipRows: data.skipRows,
      useCols: useCols
    });

    if (JSON.stringify(result).startsWith("cat failed:")) {
      throw JSON.stringify(result).toString();
    }

    result.value = res;
    completed.value = true;
  } catch (err) {
    ElNotification({
      title: "Cat failed",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
  isLoading.value = false;
}

const { compiledMarkdown, manualHighlight } = useMarkdown(catContent);
const handleDialogOpened = async () => {
  await nextTick();
  manualHighlight();
};
</script>

<template>
  <el-form class="page-container" :style="formHeight">
    <el-form>
      <div class="custom-container1">
        <div class="custom-container2">
          <el-button @click="selectFile()" :icon="FolderOpened" plain>
            Open File
          </el-button>

          <el-tooltip
            content="Polars memory or stream, Csv stream Cat"
            placement="top"
            effect="light"
          >
            <el-select
              v-model="data.mode"
              style="margin-left: 10px; width: 100px"
            >
              <el-option label="Memory" value="memory" />
              <el-option label="Stream" value="stream" />
              <el-option label="Csv" value="csv" />
            </el-select>
          </el-tooltip>

          <el-tooltip content="skip rows" placement="top" effect="light">
            <el-input
              v-model="data.skipRows"
              style="margin-left: 10px; width: 80px"
              placeholder="skip rows"
            />
          </el-tooltip>

          <el-button
            @click="concatData()"
            :loading="isLoading"
            :icon="Connection"
            plain
            style="margin-left: 10px"
          >
            Cat
          </el-button>
        </div>

        <el-link @click="infoDialog = true" :icon="Link">
          <span v-if="completed">
            <el-icon color="green" style="margin-right: 2px">
              <Check />
            </el-icon>
            Cat done, elapsed time: {{ result }} s
          </span>
          <span v-else>
            How to use
            <span style="color: skyblue; font-weight: bold">cat</span>
          </span>
        </el-link>
      </div>
    </el-form>

    <el-select
      v-model="columns"
      multiple
      filterable
      style="margin-top: 12px; width: 100%"
      placeholder="Choose columns if you need to cat specific column"
    >
      <el-option
        v-for="item in originalColumns"
        :key="item.value"
        :label="item.label"
        :value="item.value"
      />
    </el-select>

    <el-table
      ref="tableRef"
      :data="selectedFiles"
      :height="formHeight"
      empty-text=""
      style="width: 100%"
    >
      <el-table-column type="index" width="50" />
      <el-table-column prop="filename" />
    </el-table>

    <el-dialog
      v-model="infoDialog"
      title="Cat - Merge multiple CSV or Excel files into one CSV or xlsx file"
      width="800"
      @opened="handleDialogOpened"
    >
      <el-scrollbar :height="formHeight * 0.8">
        <div v-html="compiledMarkdown" />
      </el-scrollbar>
    </el-dialog>
  </el-form>
</template>
