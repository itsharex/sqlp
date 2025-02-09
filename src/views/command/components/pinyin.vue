<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElNotification } from "element-plus";
import { FolderOpened, SwitchFilled } from "@element-plus/icons-vue";
import { useDynamicFormHeight } from "@/utils/utils";
import { viewOpenFile, viewSqlp } from "@/utils/view";

const [isLoading, isPath, columns, tableHeader, tableColumn, tableData] = [
  ref(false),
  ref(false),
  ref(""),
  ref([]),
  ref([]),
  ref([])
];
const data = reactive({
  path: "",
  skipRows: "0"
});
const { formHeight } = useDynamicFormHeight(190);

async function selectFile() {
  isPath.value = false;
  columns.value = "";
  tableHeader.value = [];
  tableColumn.value = [];
  tableData.value = [];

  data.path = await viewOpenFile(false, "csv", ["*"]);
  if (data.path === null) {
    return;
  }
  isPath.value = true;

  try {
    const { headerView, columnView, dataView } = await viewSqlp(
      data.path,
      data.skipRows
    );
    tableHeader.value = headerView;
    tableColumn.value = columnView;
    tableData.value = dataView;
  } catch (err) {
    ElNotification({
      title: "Open file error",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
}

// invoke pinyin
async function chineseToPinyin() {
  if (data.path === "") {
    ElNotification({
      title: "File not found",
      message: "未选择csv文件",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }
  if (columns.value.length === 0) {
    ElNotification({
      title: "Column not found",
      message: "未选择columns",
      position: "bottom-right",
      type: "warning"
    });
    return;
  }

  const cols = Object.values(columns.value).join("|");

  isLoading.value = true;

  try {
    const result: string = await invoke("pinyin", {
      path: data.path,
      columns: cols,
      skipRows: data.skipRows
    });

    if (JSON.stringify(result).startsWith("pinyin failed:")) {
      throw JSON.stringify(result).toString();
    }

    ElNotification({
      message: `Convert done, elapsed time: ${result} s`,
      position: "bottom-right",
      type: "success",
      duration: 5000
    });
  } catch (err) {
    ElNotification({
      title: "Pinyin failed",
      message: err.toString(),
      position: "bottom-right",
      type: "error",
      duration: 10000
    });
  }
  isLoading.value = false;
}
</script>

<template>
  <div class="page-container">
    <div class="custom-container1">
      <div class="custom-container2">
        <el-button @click="selectFile()" :icon="FolderOpened" plain>
          Open File
        </el-button>

        <el-tooltip content="skip rows" placement="top" effect="light">
          <el-input
            v-model="data.skipRows"
            style="margin-left: 10px; width: 50px"
            placeholder="skip rows"
          />
        </el-tooltip>

        <el-button
          @click="chineseToPinyin()"
          :loading="isLoading"
          :icon="SwitchFilled"
          plain
          style="margin-left: 10px"
        >
          Convert
        </el-button>
      </div>

      <el-text>
        <span v-if="isPath">{{ data.path }}</span>
        <span v-else>Convert Chinese to Pinyin in CSV</span>
      </el-text>
    </div>

    <el-select
      v-model="columns"
      multiple
      filterable
      style="margin-top: 12px; width: 100%"
      placeholder="please choose columns"
    >
      <el-option
        v-for="item in tableHeader"
        :key="item.value"
        :label="item.label"
        :value="item.value"
      />
    </el-select>

    <el-table
      :data="tableData"
      :height="formHeight"
      border
      empty-text=""
      style="margin-top: 12px; width: 100%"
    >
      <el-table-column
        v-for="column in tableColumn"
        :prop="column.prop"
        :label="column.label"
        :key="column.prop"
      />
    </el-table>
  </div>
</template>
