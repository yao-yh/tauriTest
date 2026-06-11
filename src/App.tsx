import { useCallback, useEffect, useState } from 'react'
import {
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  ReloadOutlined,
} from '@ant-design/icons'
import { invoke } from '@tauri-apps/api/core'
import {
  App as AntdApp,
  Button,
  Card,
  ConfigProvider,
  Empty,
  Form,
  Input,
  Modal,
  Popconfirm,
  Space,
  Table,
  Tag,
  Typography,
  theme,
} from 'antd'
import type { TableProps } from 'antd'
import zhCN from 'antd/locale/zh_CN'
import './App.css'

interface Item {
  id: number
  name: string
  description: string | null
  createdAt: string
  updatedAt: string
}

interface ItemForm {
  name: string
  description?: string
}

function formatDate(value: string) {
  const date = new Date(value.replace(' ', 'T') + 'Z')
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString('zh-CN')
}

function CrudPage() {
  const { message } = AntdApp.useApp()
  const [form] = Form.useForm<ItemForm>()
  const [items, setItems] = useState<Item[]>([])
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [editingItem, setEditingItem] = useState<Item | null>(null)
  const [modalOpen, setModalOpen] = useState(false)

  const loadItems = useCallback(async () => {
    setLoading(true)
    try {
      setItems(await invoke<Item[]>('list_items'))
    } catch (error) {
      message.error(`读取数据失败：${String(error)}`)
    } finally {
      setLoading(false)
    }
  }, [message])

  useEffect(() => {
    void loadItems()
  }, [loadItems])

  const openCreateModal = () => {
    setEditingItem(null)
    form.resetFields()
    setModalOpen(true)
  }

  const openEditModal = (item: Item) => {
    setEditingItem(item)
    form.setFieldsValue({
      name: item.name,
      description: item.description ?? undefined,
    })
    setModalOpen(true)
  }

  const saveItem = async () => {
    const values = await form.validateFields()
    setSaving(true)
    try {
      if (editingItem) {
        await invoke('update_item', { id: editingItem.id, input: values })
        message.success('记录已更新')
      } else {
        await invoke('create_item', { input: values })
        message.success('记录已添加')
      }
      setModalOpen(false)
      await loadItems()
    } catch (error) {
      message.error(`保存失败：${String(error)}`)
    } finally {
      setSaving(false)
    }
  }

  const deleteItem = async (id: number) => {
    try {
      await invoke('delete_item', { id })
      message.success('记录已删除')
      await loadItems()
    } catch (error) {
      message.error(`删除失败：${String(error)}`)
    }
  }

  const columns: TableProps<Item>['columns'] = [
    {
      title: '编号',
      dataIndex: 'id',
      width: 90,
      render: (id: number) => <Tag color="blue">#{id}</Tag>,
    },
    {
      title: '名称',
      dataIndex: 'name',
      ellipsis: true,
      render: (name: string) => <Typography.Text strong>{name}</Typography.Text>,
    },
    {
      title: '说明',
      dataIndex: 'description',
      ellipsis: true,
      render: (description: string | null) => description || <span className="muted">暂无说明</span>,
    },
    {
      title: '更新时间',
      dataIndex: 'updatedAt',
      width: 190,
      render: formatDate,
    },
    {
      title: '操作',
      key: 'actions',
      width: 150,
      render: (_, item) => (
        <Space>
          <Button type="link" icon={<EditOutlined />} onClick={() => openEditModal(item)}>
            编辑
          </Button>
          <Popconfirm
            title="删除这条记录？"
            description="删除后无法恢复。"
            okText="删除"
            cancelText="取消"
            okButtonProps={{ danger: true }}
            onConfirm={() => deleteItem(item.id)}
          >
            <Button type="link" danger icon={<DeleteOutlined />}>
              删除
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ]

  return (
    <main className="page-shell">
      <section className="hero">
        <div>
          <Typography.Title level={2}>桌面列表管理</Typography.Title>
          <Typography.Paragraph>
            Tauri + React 19 + Ant Design + Prisma SQLite
          </Typography.Paragraph>
        </div>
        <Tag color="green">本地数据库持久化</Tag>
      </section>

      <Card className="content-card" bordered={false}>
        <div className="toolbar">
          <div>
            <Typography.Title level={4}>数据列表</Typography.Title>
            <Typography.Text type="secondary">共 {items.length} 条记录</Typography.Text>
          </div>
          <Space>
            <Button icon={<ReloadOutlined />} onClick={() => void loadItems()}>
              刷新
            </Button>
            <Button type="primary" icon={<PlusOutlined />} onClick={openCreateModal}>
              新增记录
            </Button>
          </Space>
        </div>

        <Table<Item>
          rowKey="id"
          columns={columns}
          dataSource={items}
          loading={loading}
          pagination={{ pageSize: 8, showSizeChanger: false }}
          locale={{ emptyText: <Empty description="还没有数据，点击右上角新增" /> }}
          scroll={{ x: 800 }}
        />
      </Card>

      <Modal
        title={editingItem ? '编辑记录' : '新增记录'}
        open={modalOpen}
        confirmLoading={saving}
        okText="保存"
        cancelText="取消"
        destroyOnHidden
        onOk={() => void saveItem()}
        onCancel={() => setModalOpen(false)}
      >
        <Form form={form} layout="vertical" requiredMark="optional">
          <Form.Item
            label="名称"
            name="name"
            rules={[
              { required: true, whitespace: true, message: '请输入名称' },
              { max: 100, message: '名称最多 100 个字符' },
            ]}
          >
            <Input placeholder="例如：第一条记录" autoFocus />
          </Form.Item>
          <Form.Item
            label="说明"
            name="description"
            rules={[{ max: 500, message: '说明最多 500 个字符' }]}
          >
            <Input.TextArea rows={4} placeholder="可选，补充一些说明" showCount maxLength={500} />
          </Form.Item>
        </Form>
      </Modal>
    </main>
  )
}

function App() {
  return (
    <ConfigProvider
      locale={zhCN}
      theme={{
        algorithm: theme.defaultAlgorithm,
        token: {
          colorPrimary: '#1677ff',
          borderRadius: 10,
          fontFamily: '"Segoe UI", "Microsoft YaHei", sans-serif',
        },
      }}
    >
      <AntdApp>
        <CrudPage />
      </AntdApp>
    </ConfigProvider>
  )
}

export default App
